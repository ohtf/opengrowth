// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-FileCopyrightText: Royal College of Paediatrics and Child Health
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The Turner syndrome, UK Trisomy 21, and AAP Trisomy 21 references: which
//! table serves an age, and where there is none.
//!
//! Ported from `rcpchgrowth.turner`, `trisomy_21`, and `trisomy_21_aap`.
//! Each absence message is upstream's `LookupError` text, which reaches API
//! responses unchanged.

use crate::data::TableId;
use crate::vocabulary::{MeasurementMethod, Sex};

/// No Turner syndrome data exists for this request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum TurnerAbsence {
    /// Below 1 year.
    #[error("There is no reference data below 1 year.")]
    BelowOneYear,
    /// Above 20 years.
    #[error("There is no reference data above 20 years.")]
    AboveTwentyYears,
    /// Weight.
    #[error("There is no reference data for weight.")]
    Weight,
    /// BMI.
    #[error("There is no reference data for BMI (body mass index).")]
    Bmi,
    /// Head circumference.
    #[error("There is no reference data for head circumference.")]
    Ofc,
    /// A boy.
    #[error("Turner's syndrome only affects girls and women.")]
    Male,
}

/// The Turner table for a request (upstream `reference_data_absent` and
/// `turner_lms_array_for_measurement_and_sex`): girls' height, 1 to 20 years.
///
/// # Errors
///
/// Where there is no Turner data, with upstream's message.
pub fn turner(age: f64, method: MeasurementMethod, sex: Sex) -> Result<TableId, TurnerAbsence> {
    if age < 1.0 {
        Err(TurnerAbsence::BelowOneYear)
    } else if age > 20.0 {
        Err(TurnerAbsence::AboveTwentyYears)
    } else if method == MeasurementMethod::Weight {
        Err(TurnerAbsence::Weight)
    } else if method == MeasurementMethod::Bmi {
        Err(TurnerAbsence::Bmi)
    } else if method == MeasurementMethod::Ofc {
        Err(TurnerAbsence::Ofc)
    } else if sex == Sex::Male {
        Err(TurnerAbsence::Male)
    } else {
        Ok(TableId::Turner)
    }
}

/// No UK Trisomy 21 data exists for this request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum Trisomy21Absence {
    /// Below term.
    #[error("No reference data exists below 40 weeks gestation")]
    BelowTerm,
    /// Above 20 years.
    #[error("Trisomy 21 reference data does not exist over the age of 20y.")]
    AboveTwentyYears,
    /// BMI above 18.82 years. The table runs to 18.92, so two rows per sex
    /// are unreachable (`spec/standard.md` section 6.2).
    #[error("Trisomy BMI reference data does not exist > 18.82 y.")]
    BmiAboveLimit,
    /// Head circumference above 18 years.
    #[error("Trisomy head circumference reference data does not exist > 18 y")]
    OfcAboveEighteenYears,
}

/// The UK Trisomy 21 table for a request (upstream `trisomy_21`).
///
/// # Errors
///
/// Where there is no Trisomy 21 data, with upstream's message.
pub fn trisomy_21(age: f64, method: MeasurementMethod) -> Result<TableId, Trisomy21Absence> {
    if age < 0.0 {
        Err(Trisomy21Absence::BelowTerm)
    } else if age > 20.0 {
        Err(Trisomy21Absence::AboveTwentyYears)
    } else if method == MeasurementMethod::Bmi && age > 18.82 {
        Err(Trisomy21Absence::BmiAboveLimit)
    } else if method == MeasurementMethod::Ofc && age > 18.0 {
        Err(Trisomy21Absence::OfcAboveEighteenYears)
    } else {
        Ok(TableId::Trisomy21)
    }
}

/// The first age in the AAP infant table for height and head circumference.
pub const AAP_INFANT_FIRST_AGE: f64 = 0.083_333_333;

/// No AAP Trisomy 21 data exists for this request. Upstream prefixes each
/// message with "Reference data absent: ".
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum Trisomy21AapAbsence {
    /// Below term.
    #[error(
        "Reference data absent: No Trisomy 21 (AAP) reference data exists below 40 weeks gestation"
    )]
    BelowTerm,
    /// Height or head circumference below 1 month.
    #[error(
        "Reference data absent: No Trisomy 21 (AAP) reference data exists below 1 month of age for height or head circumference."
    )]
    BelowOneMonth,
    /// BMI below 2 years.
    #[error(
        "Reference data absent: No Trisomy 21 (AAP) reference data exists below 2 years of age for BMI."
    )]
    BmiBelowTwoYears,
    /// Above 20 years.
    #[error("Reference data absent: Trisomy 21 reference data does not exist over the age of 20y.")]
    AboveTwentyYears,
}

/// The AAP Trisomy 21 table for a request (upstream `trisomy_21_aap`).
/// `youngest_at_boundary` chooses the infant table for head circumference
/// at exactly 3 years; height and weight use it at 3 years regardless.
///
/// # Errors
///
/// Where there is no AAP Trisomy 21 data, with upstream's message.
#[allow(
    clippy::float_cmp,
    reason = "upstream compares the boundary age exactly"
)]
pub fn trisomy_21_aap(
    age: f64,
    method: MeasurementMethod,
    youngest_at_boundary: bool,
) -> Result<TableId, Trisomy21AapAbsence> {
    use MeasurementMethod::{Bmi, Height, Ofc, Weight};
    if age < 0.0 {
        return Err(Trisomy21AapAbsence::BelowTerm);
    }
    if age < AAP_INFANT_FIRST_AGE && matches!(method, Height | Ofc) {
        return Err(Trisomy21AapAbsence::BelowOneMonth);
    }
    if age < 2.0 && method == Bmi {
        return Err(Trisomy21AapAbsence::BmiBelowTwoYears);
    }
    if age > 20.0 {
        return Err(Trisomy21AapAbsence::AboveTwentyYears);
    }
    if age <= 3.0 && matches!(method, Height | Weight | Ofc) {
        if method == Ofc && age == 3.0 && !youngest_at_boundary {
            return Ok(TableId::Trisomy21AapChild);
        }
        return Ok(TableId::Trisomy21AapInfant);
    }
    Ok(TableId::Trisomy21AapChild)
}

#[cfg(test)]
mod tests {
    use super::*;
    use MeasurementMethod::{Bmi, Height, Ofc, Weight};

    #[test]
    fn turner_is_girls_height_from_one_to_twenty() {
        assert_eq!(turner(5.0, Height, Sex::Female), Ok(TableId::Turner));
        assert_eq!(
            turner(0.5, Height, Sex::Female),
            Err(TurnerAbsence::BelowOneYear)
        );
        assert_eq!(turner(5.0, Bmi, Sex::Male), Err(TurnerAbsence::Bmi));
        assert_eq!(turner(5.0, Height, Sex::Male), Err(TurnerAbsence::Male));
    }

    #[test]
    fn trisomy_21_limits_follow_upstream() {
        assert_eq!(trisomy_21(18.9, Bmi), Err(Trisomy21Absence::BmiAboveLimit));
        assert_eq!(
            trisomy_21(18.5, Ofc),
            Err(Trisomy21Absence::OfcAboveEighteenYears)
        );
        assert_eq!(trisomy_21(17.0, Weight), Ok(TableId::Trisomy21));
    }

    #[test]
    fn aap_switches_tables_at_three_years() {
        assert_eq!(
            trisomy_21_aap(3.0, Height, false),
            Ok(TableId::Trisomy21AapInfant)
        );
        assert_eq!(
            trisomy_21_aap(3.0, Ofc, false),
            Ok(TableId::Trisomy21AapChild)
        );
        assert_eq!(
            trisomy_21_aap(3.0, Ofc, true),
            Ok(TableId::Trisomy21AapInfant)
        );
        assert_eq!(
            trisomy_21_aap(2.5, Bmi, false),
            Ok(TableId::Trisomy21AapChild)
        );
        assert_eq!(
            trisomy_21_aap(0.05, Weight, false),
            Ok(TableId::Trisomy21AapInfant)
        );
        assert_eq!(
            trisomy_21_aap(0.05, Ofc, false),
            Err(Trisomy21AapAbsence::BelowOneMonth)
        );
    }
}
