// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-FileCopyrightText: Royal College of Paediatrics and Child Health
// SPDX-License-Identifier: AGPL-3.0-or-later

//! SDS and centile for a single measurement at a given decimal age.
//!
//! Ported from `rcpchgrowth.global_functions.sds_for_measurement` and
//! `centile`. Each reference's [`Definition`] chooses the table, how lookups
//! interpolate, and the method:
//!
//! - **LMS**, for most references.
//! - **WHO's restricted LMS method**: beyond 3 SD (for weight and BMI at any
//!   age, and for everything under 5 years), SDS beyond 3 are spaced by the
//!   2-to-3 SD distance. WHO's SDS lookups also interpolate linearly.
//! - **The CDC extended BMI-for-age method**: above the 95th centile, the
//!   centile is `90 + 10 * Phi((BMI - p95) / sigma)`, with `sigma` from the
//!   table. Its tables carry sigma, so their lookups are always linear.

use crate::data::{DataError, ReferenceData, TableId};
use crate::lms::{Lms, measurement_from_lms, sds_from_lms};
use crate::normal;
use crate::references::{self, Absence, Definition, SdsMethod};
use crate::round::PyFloat;
use crate::table::{Interpolation, LmsTable, Lookup, TableError};
use crate::vocabulary::{GrowthReference, MeasurementMethod, Sex};

/// Why an SDS could not be calculated.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum CalcError {
    /// The reference has no data for this age or measurement. Upstream this
    /// is a `LookupError`, and its message reaches API responses verbatim.
    #[error(transparent)]
    Absent(#[from] Absence),
    /// The LMS transform has no real value here. Upstream computes a complex
    /// number and fails when it rounds it.
    #[error("the LMS transform has no real value at this SDS")]
    NotReal,
    /// CDC BMI needs the extended-BMI sigma, and this table has none: the
    /// earlier WHO infant table, which a chart reaches at exactly 2 years.
    /// The message is Python's `KeyError` text.
    #[error("'sigma'")]
    MissingSigma,
    /// A table needed for this calculation is not loaded.
    #[error(transparent)]
    Data(#[from] DataError),
    /// The age could not be looked up in the table.
    #[error(transparent)]
    Table(#[from] TableError),
    /// The observation is zero, negative, or not finite. Upstream produces a
    /// complex number or a crash here (a Category 3 defect, `spec/standard.md` section 6.3).
    #[error("observation value must be a positive number, got {0}")]
    InvalidObservation(f64),
}

/// An SDS, with the reference and the table it was calculated from.
///
/// Carrying both makes provenance report what was computed rather than
/// what was requested (`spec/provenance.md` invariant 3).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SdsResult {
    /// The reference whose definition chose the table.
    pub growth_reference: GrowthReference,
    /// The SDS (z-score). Unrounded.
    pub sds: f64,
    /// The source table the LMS values came from.
    pub table: TableId,
    /// The LMS values used, exact or interpolated.
    pub lms: Lms,
}

/// SDS for an observation at a decimal age.
///
/// # Errors
///
/// See [`CalcError`].
pub fn sds_for_measurement(
    data: &dyn ReferenceData,
    reference: GrowthReference,
    age: f64,
    method: MeasurementMethod,
    observation: f64,
    sex: Sex,
) -> Result<SdsResult, CalcError> {
    let definition = references::definition(reference);
    let (table, lms, _) = lms_for_age(
        data,
        definition,
        age,
        method,
        sex,
        false,
        definition.sds.interpolation,
        || {
            // Checked after the age rules so the reported error matches upstream.
            if observation.is_finite() && observation > 0.0 {
                Ok(())
            } else {
                Err(CalcError::InvalidObservation(observation))
            }
        },
    )?;
    let cdc_bmi =
        definition.sds.method == SdsMethod::CdcExtendedBmi && method == MeasurementMethod::Bmi;
    if cdc_bmi {
        let sigma = lms.sigma.ok_or(CalcError::MissingSigma)?;
        let p95 = cdc_p95(&lms);
        if observation > p95 {
            let centile = normal::cdf((observation - p95) / sigma) * 10.0 + 90.0;
            let sds = normal::ppf(centile / 100.0);
            return Ok(SdsResult {
                growth_reference: definition.reference,
                sds,
                table,
                lms,
            });
        }
    }
    let bmi_or_weight = matches!(method, MeasurementMethod::Bmi | MeasurementMethod::Weight);
    let restricted = definition.sds.method == SdsMethod::WhoRestricted;
    let sds = if restricted && ((bmi_or_weight && age >= 5.0) || age < 5.0) {
        who_sds(observation, &lms)
    } else {
        sds_from_lms(observation, &lms)
    };
    Ok(SdsResult {
        growth_reference: definition.reference,
        sds,
        table,
        lms,
    })
}

/// The BMI at the 95th centile, where the CDC extended method takes over.
fn cdc_p95(lms: &Lms) -> f64 {
    lms.m * (1.0 + lms.l * lms.s * 1.645).powf(1.0 / lms.l)
}

/// WHO's restricted extension of the LMS method beyond 3 SD: the distance
/// from 3 SD is measured in units of the distance between 2 and 3 SD.
fn who_sds(observation: f64, lms: &Lms) -> f64 {
    let z = sds_from_lms(observation, lms);
    if z.abs() > 3.0 {
        let at = |sds: f64| lms.m * (1.0 + lms.l * lms.s * sds).powf(1.0 / lms.l);
        if z > 3.0 {
            let (sd3, sd2) = (at(3.0), at(2.0));
            return 3.0 + ((observation - sd3) / (sd3 - sd2));
        } else if z < -3.0 {
            let (sd3, sd2) = (at(-3.0), at(-2.0));
            return -3.0 + ((observation - sd3) / (sd2 - sd3));
        }
    }
    z
}

/// The measurement at an SDS for an age (the upstream `measurement_from_sds`),
/// rounded to four decimal places as upstream does, with the table it came
/// from. The measurement is `None` where the LMS transform has no real
/// answer, which upstream also returns as `None`. It keeps its kind, because
/// callers round it again.
///
/// The rounding depends on whether the upstream result is a NumPy `float64`
/// ([`PyFloat`]). It is if the LMS values were linearly interpolated, or if
/// `requested_sds` is one and `L` is not 0; when `L` is 0, `math.exp`
/// returns a Python float. Pass `PyFloat::Builtin` for an SDS that is a
/// plain number.
///
/// `youngest_at_boundary` selects the younger table at a join between two
/// segments, so a chart line runs right up to the join.
///
/// # Errors
///
/// As for [`sds_for_measurement`], when there is no reference data at this age.
pub fn measurement_from_sds(
    data: &dyn ReferenceData,
    reference: GrowthReference,
    requested_sds: PyFloat,
    method: MeasurementMethod,
    sex: Sex,
    age: f64,
    youngest_at_boundary: bool,
) -> Result<(Option<PyFloat>, TableId), CalcError> {
    let definition = references::definition(reference);
    let (table_id, lms, lookup) = lms_for_age(
        data,
        definition,
        age,
        method,
        sex,
        youngest_at_boundary,
        Interpolation::CubicWherePossible,
        || Ok(()),
    )?;
    let z = requested_sds.value();
    if definition.sds.method == SdsMethod::CdcExtendedBmi && method == MeasurementMethod::Bmi {
        let sigma = lms.sigma.ok_or(CalcError::MissingSigma)?;
        // Upstream's CDC BMI branch, which does not check for a negative base.
        let value = if z <= 1.645 {
            let numpy = lookup == Lookup::Linear || matches!(requested_sds, PyFloat::NumPy(_));
            let value = lms.m * (1.0 + lms.l * lms.s * z).powf(1.0 / lms.l);
            if numpy {
                PyFloat::NumPy(value)
            } else {
                PyFloat::Builtin(value)
            }
        } else {
            let centile = normal::cdf(z) * 100.0;
            PyFloat::NumPy(normal::ppf((centile - 90.0) / 10.0) * sigma + cdc_p95(&lms))
        };
        if !value.value().is_finite() {
            return Err(CalcError::NotReal);
        }
        return Ok((Some(value.round(4)), table_id));
    }
    let numpy =
        lookup == Lookup::Linear || (matches!(requested_sds, PyFloat::NumPy(_)) && lms.l != 0.0);
    let measurement = measurement_from_lms(z, &lms).map(|value| {
        let value = if numpy {
            PyFloat::NumPy(value)
        } else {
            PyFloat::Builtin(value)
        };
        value.round(4)
    });
    Ok((measurement, table_id))
}

/// A child's BMI as a percentage of the median BMI for age and sex (the
/// upstream `percentage_median_bmi`), with the table it came from.
///
/// # Errors
///
/// As for [`sds_for_measurement`], when there is no BMI reference at this age.
pub fn percentage_median_bmi(
    data: &dyn ReferenceData,
    reference: GrowthReference,
    age: f64,
    bmi: f64,
    sex: Sex,
) -> Result<(f64, TableId), CalcError> {
    let (table, lms, _) = lms_for_age(
        data,
        references::definition(reference),
        age,
        MeasurementMethod::Bmi,
        sex,
        false,
        Interpolation::CubicWherePossible,
        || Ok(()),
    )?;
    Ok(((bmi / lms.m) * 100.0, table))
}

/// Selects the table for `age` and looks up its LMS values (the upstream
/// `lms_value_array_for_measurement_for_reference` and `fetch_lms`).
/// `before_lookup` runs after the reference's own age rules, so callers can
/// add checks without changing which error upstream would report first.
#[allow(clippy::too_many_arguments, reason = "the upstream lookup's inputs")]
fn lms_for_age(
    data: &dyn ReferenceData,
    definition: &Definition,
    age: f64,
    method: MeasurementMethod,
    sex: Sex,
    youngest_at_boundary: bool,
    interpolation: Interpolation,
    before_lookup: impl FnOnce() -> Result<(), CalcError>,
) -> Result<(TableId, Lms, Lookup), CalcError> {
    let table_id = (definition.select)(age, method, sex, youngest_at_boundary)?;
    before_lookup()?;
    let table = data.table(table_id, method, sex)?;
    let (lms, lookup) = table.lookup(age_for_lms_lookup(definition, age, table), interpolation)?;
    Ok((table_id, lms, lookup))
}

/// Centile (0 to 100) for an SDS.
#[must_use]
pub fn centile(sds: f64) -> f64 {
    normal::cdf(sds) * 100.0
}

/// The upstream `_age_for_lms_lookup`: at exactly two years, UK-WHO and WHO
/// clamp the age into the selected daily table, whose first row is one day
/// after the second birthday.
fn age_for_lms_lookup(definition: &Definition, age: f64, table: &LmsTable) -> f64 {
    if !definition.sds.clamp_at_two_years || age != 2.0 {
        return age;
    }
    age.max(table.first_age()).min(table.last_age())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::table::LmsPoint;

    /// One table, served for whichever source is asked for.
    struct Fixed(LmsTable);

    impl ReferenceData for Fixed {
        fn table(&self, _: TableId, _: MeasurementMethod, _: Sex) -> Result<&LmsTable, DataError> {
            Ok(&self.0)
        }
    }

    struct Nothing;

    impl ReferenceData for Nothing {
        fn table(
            &self,
            table: TableId,
            _: MeasurementMethod,
            _: Sex,
        ) -> Result<&LmsTable, DataError> {
            Err(DataError::Unavailable { table })
        }
    }

    fn flat_table(first: f64, last: f64) -> Fixed {
        let points = (0..=10)
            .map(|i| LmsPoint {
                age: first + (last - first) * f64::from(i) / 10.0,
                l: 1.0,
                m: 100.0,
                s: 0.1,
                sigma: None,
            })
            .collect();
        Fixed(LmsTable::new(points).unwrap())
    }

    #[test]
    fn median_is_zero_sds_and_fiftieth_centile() {
        let data = flat_table(2.002, 5.0);
        let result = sds_for_measurement(
            &data,
            GrowthReference::UkWho,
            3.0,
            MeasurementMethod::Height,
            100.0,
            Sex::Male,
        )
        .unwrap();
        assert_eq!(result.sds, 0.0);
        assert_eq!(result.table, TableId::Who2006Child);
        assert_eq!(centile(result.sds), 50.0);
    }

    #[test]
    fn exactly_two_years_clamps_into_the_child_table() {
        let data = flat_table(2.002, 5.0);
        let result = sds_for_measurement(
            &data,
            GrowthReference::UkWho,
            2.0,
            MeasurementMethod::Height,
            110.0,
            Sex::Female,
        )
        .unwrap();
        assert!((result.sds - 1.0).abs() < 1e-12);
    }

    #[test]
    fn missing_tables_are_reported_not_substituted() {
        let err = sds_for_measurement(
            &Nothing,
            GrowthReference::UkWho,
            10.0,
            MeasurementMethod::Weight,
            30.0,
            Sex::Male,
        )
        .unwrap_err();
        assert_eq!(
            err,
            CalcError::Data(DataError::Unavailable {
                table: TableId::Uk90Child
            })
        );
    }

    #[test]
    fn absence_messages_are_the_upstream_strings() {
        let err = sds_for_measurement(
            &Nothing,
            GrowthReference::UkWho,
            0.0,
            MeasurementMethod::Bmi,
            13.0,
            Sex::Male,
        )
        .unwrap_err();
        assert_eq!(
            err.to_string(),
            "UK-WHO BMI data does not exist below 2 weeks of age."
        );
    }

    #[test]
    fn invalid_observations_and_absent_data_are_errors() {
        let data = flat_table(2.002, 5.0);
        assert!(matches!(
            sds_for_measurement(
                &data,
                GrowthReference::UkWho,
                3.0,
                MeasurementMethod::Height,
                -1.0,
                Sex::Male
            ),
            Err(CalcError::InvalidObservation(_))
        ));
        assert!(matches!(
            sds_for_measurement(
                &data,
                GrowthReference::TurnersSyndrome,
                3.0,
                MeasurementMethod::Height,
                90.0,
                Sex::Male
            ),
            Err(CalcError::Absent(crate::references::turner::MALE))
        ));
    }
}
