// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-FileCopyrightText: Royal College of Paediatrics and Child Health
// SPDX-License-Identifier: AGPL-3.0-or-later

//! SDS and centile for a single measurement at a given decimal age.
//!
//! Ported from `rcpchgrowth.global_functions.sds_for_measurement` and
//! `centile`. Only UK-WHO is implemented so far (`R9`); other references
//! return [`CalcError::ReferenceNotImplemented`] rather than a wrong answer.

use crate::data::{DataError, ReferenceData, TableId};
use crate::lms::{Lms, measurement_from_lms, sds_from_lms};
use crate::normal;
use crate::round::PyFloat;
use crate::table::{Interpolation, LmsTable, Lookup, TableError};
use crate::uk_who::{self, UkWhoAbsence};
use crate::vocabulary::{GrowthReference, MeasurementMethod, Sex};

/// Why an SDS could not be calculated.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum CalcError {
    /// The reference has no data for this age or measurement. Upstream this
    /// is a `LookupError`, and its message reaches API responses verbatim.
    #[error(transparent)]
    UkWhoAbsent(#[from] UkWhoAbsence),
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
    /// The reference has not been ported yet.
    #[error("the {0} reference is not implemented yet")]
    ReferenceNotImplemented(GrowthReference),
}

/// An SDS together with the source table it was computed from.
///
/// Carrying the table makes provenance report what was computed rather than
/// what was requested (`spec/provenance.md` invariant 3).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SdsResult {
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
    let (table, lms) = lms_for_age(data, reference, age, method, sex, || {
        // Checked after the age rules so the reported error matches upstream.
        if observation.is_finite() && observation > 0.0 {
            Ok(())
        } else {
            Err(CalcError::InvalidObservation(observation))
        }
    })?;
    Ok(SdsResult {
        sds: sds_from_lms(observation, &lms),
        table,
        lms,
    })
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
    let (lms, lookup, table_id) = match reference {
        GrowthReference::UkWho => {
            let table_id = uk_who::table_for_age(age, youngest_at_boundary)?;
            if let Some(absence) = uk_who::data_absent(age, method, sex) {
                return Err(absence.into());
            }
            let table = data.table(table_id, method, sex)?;
            let (lms, lookup) = table.lookup(
                age_for_lms_lookup(reference, age, table),
                Interpolation::CubicWherePossible,
            )?;
            (lms, lookup, table_id)
        }
        other => return Err(CalcError::ReferenceNotImplemented(other)),
    };
    let numpy =
        lookup == Lookup::Linear || (matches!(requested_sds, PyFloat::NumPy(_)) && lms.l != 0.0);
    let measurement = measurement_from_lms(requested_sds.value(), &lms).map(|value| {
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
    let (table, lms) = lms_for_age(data, reference, age, MeasurementMethod::Bmi, sex, || Ok(()))?;
    Ok(((bmi / lms.m) * 100.0, table))
}

/// Selects the table for `age` and looks up its LMS values (the upstream
/// `lms_value_array_for_measurement_for_reference` and `fetch_lms`).
/// `before_lookup` runs after the reference's own age rules, so callers can
/// add checks without changing which error upstream would report first.
fn lms_for_age(
    data: &dyn ReferenceData,
    reference: GrowthReference,
    age: f64,
    method: MeasurementMethod,
    sex: Sex,
    before_lookup: impl FnOnce() -> Result<(), CalcError>,
) -> Result<(TableId, Lms), CalcError> {
    match reference {
        GrowthReference::UkWho => {
            let table_id = uk_who::table_for_age(age, false)?;
            if let Some(absence) = uk_who::data_absent(age, method, sex) {
                return Err(absence.into());
            }
            before_lookup()?;
            let table = data.table(table_id, method, sex)?;
            let lms = table.lms_at(
                age_for_lms_lookup(reference, age, table),
                Interpolation::CubicWherePossible,
            )?;
            Ok((table_id, lms))
        }
        other => Err(CalcError::ReferenceNotImplemented(other)),
    }
}

/// Centile (0 to 100) for an SDS.
#[must_use]
pub fn centile(sds: f64) -> f64 {
    normal::cdf(sds) * 100.0
}

/// The upstream `_age_for_lms_lookup`: at exactly two years, UK-WHO and WHO
/// clamp the age into the selected daily table, whose first row is one day
/// after the second birthday.
fn age_for_lms_lookup(reference: GrowthReference, age: f64, table: &LmsTable) -> f64 {
    let applies = matches!(reference, GrowthReference::UkWho | GrowthReference::Who);
    if !applies || age != 2.0 {
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
    fn invalid_observations_and_unported_references_are_errors() {
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
                GrowthReference::Cdc,
                3.0,
                MeasurementMethod::Height,
                90.0,
                Sex::Male
            ),
            Err(CalcError::ReferenceNotImplemented(GrowthReference::Cdc))
        ));
    }
}
