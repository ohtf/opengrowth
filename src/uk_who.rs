// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-FileCopyrightText: Royal College of Paediatrics and Child Health
// SPDX-License-Identifier: AGPL-3.0-or-later

//! UK-WHO: choosing which source table applies at a given age.
//!
//! UK-WHO stitches together three sources (ported from `rcpchgrowth.uk_who`):
//!
//! | Decimal age            | Source                         |
//! |------------------------|--------------------------------|
//! | 23 weeks to < 42 weeks | UK90 preterm                   |
//! | 42 weeks to < 2 years  | WHO 2006, measured lying       |
//! | 2 years to < 4 years   | WHO 2006, measured standing    |
//! | 4 years to 20 years    | UK90 child                     |
//!
//! At the three joins, chart generation can ask for the younger table
//! (`youngest_at_boundary`) so a curve runs right up to the join. SDS
//! calculation never does.
//!
//! Error messages are the upstream ones verbatim, because they reach API
//! responses as measurement errors.

use crate::data::TableId;
use crate::vocabulary::{MeasurementMethod, Sex};

/// Decimal age of 23 weeks gestation: the start of UK90.
pub const TWENTY_THREE_WEEKS_GESTATION: f64 = ((23 * 7) - (40 * 7)) as f64 / 365.25;
/// Decimal age of 25 weeks gestation: the start of UK90 length data.
pub const TWENTY_FIVE_WEEKS_GESTATION: f64 = ((25 * 7) - (40 * 7)) as f64 / 365.25;
/// Decimal age of 42 weeks gestation (two weeks after term): the start of WHO 2006.
pub const FORTY_TWO_WEEKS_GESTATION: f64 = ((42 * 7) - (40 * 7)) as f64 / 365.25;
/// Infants are measured standing, and use the WHO child table, from 2 years.
pub const WHO_CHILD_LOWER_THRESHOLD: f64 = 2.0;
/// UK-WHO returns to UK90 at 4 years.
pub const WHO_CHILD_UPPER_THRESHOLD: f64 = 4.0;
/// UK90 ends at 20 years.
pub const UK90_UPPER_THRESHOLD: f64 = 20.0;
/// UK90 head circumference ends at 18 years in boys.
pub const OFC_UPPER_MALE: f64 = 18.0;
/// UK90 head circumference ends at 17 years in girls.
pub const OFC_UPPER_FEMALE: f64 = 17.0;

/// No UK-WHO data exists for this request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum UkWhoAbsence {
    /// Below 23 weeks gestation.
    #[error("There is no UK90 reference data below 23 weeks gestation")]
    BelowTwentyThreeWeeks,
    /// Above 20 years.
    #[error("There is no UK90 reference data above the age of 20 years.")]
    AboveTwentyYears,
    /// Length below 25 weeks gestation.
    #[error("UK-WHO length data does not exist in infants below 25 weeks gestation.")]
    LengthBelowTwentyFiveWeeks,
    /// BMI below 2 weeks of age.
    #[error("UK-WHO BMI data does not exist below 2 weeks of age.")]
    BmiBelowTwoWeeks,
    /// Head circumference above 18 years in boys.
    #[error("UK-WHO head circumference data does not exist in boys over 18 y of age.")]
    OfcAboveEighteenMale,
    /// Head circumference above 17 years in girls.
    #[error("UK-WHO head circumference data does not exist in girls over 17 y of age.")]
    OfcAboveSeventeenFemale,
}

/// The source table for `age` (the upstream `uk_who_reference`).
///
/// # Errors
///
/// Below 23 weeks gestation or above 20 years.
pub fn table_for_age(age: f64, youngest_at_boundary: bool) -> Result<TableId, UkWhoAbsence> {
    if age < TWENTY_THREE_WEEKS_GESTATION {
        Err(UkWhoAbsence::BelowTwentyThreeWeeks)
    } else if age < FORTY_TWO_WEEKS_GESTATION {
        Ok(TableId::Uk90Preterm)
    } else if age < WHO_CHILD_LOWER_THRESHOLD {
        if age == FORTY_TWO_WEEKS_GESTATION && youngest_at_boundary {
            Ok(TableId::Uk90Preterm)
        } else {
            Ok(TableId::Who2006Infant)
        }
    } else if age < WHO_CHILD_UPPER_THRESHOLD {
        if age == WHO_CHILD_LOWER_THRESHOLD && youngest_at_boundary {
            Ok(TableId::Who2006Infant)
        } else {
            Ok(TableId::Who2006Child)
        }
    } else if age <= UK90_UPPER_THRESHOLD {
        if age == WHO_CHILD_UPPER_THRESHOLD && youngest_at_boundary {
            Ok(TableId::Who2006Child)
        } else {
            Ok(TableId::Uk90Child)
        }
    } else {
        Err(UkWhoAbsence::AboveTwentyYears)
    }
}

/// Whether UK-WHO lacks data for this measurement at this age (the upstream
/// `reference_data_absent`). Checked after [`table_for_age`], as upstream.
#[must_use]
pub fn data_absent(age: f64, method: MeasurementMethod, sex: Sex) -> Option<UkWhoAbsence> {
    // The age bounds repeat `table_for_age`, so they are unreachable in
    // sequence but kept so this function is correct on its own.
    if age < TWENTY_THREE_WEEKS_GESTATION {
        return Some(UkWhoAbsence::BelowTwentyThreeWeeks);
    }
    if age > UK90_UPPER_THRESHOLD {
        return Some(UkWhoAbsence::AboveTwentyYears);
    }
    match method {
        MeasurementMethod::Height if age < TWENTY_FIVE_WEEKS_GESTATION => {
            Some(UkWhoAbsence::LengthBelowTwentyFiveWeeks)
        }
        MeasurementMethod::Bmi if age < FORTY_TWO_WEEKS_GESTATION => {
            Some(UkWhoAbsence::BmiBelowTwoWeeks)
        }
        MeasurementMethod::Ofc => match sex {
            Sex::Male if age > OFC_UPPER_MALE => Some(UkWhoAbsence::OfcAboveEighteenMale),
            Sex::Female if age > OFC_UPPER_FEMALE => Some(UkWhoAbsence::OfcAboveSeventeenFemale),
            _ => None,
        },
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn segments_switch_at_the_documented_ages() {
        let at = |age| table_for_age(age, false);
        assert_eq!(at(TWENTY_THREE_WEEKS_GESTATION), Ok(TableId::Uk90Preterm));
        assert_eq!(at(0.0), Ok(TableId::Uk90Preterm));
        assert_eq!(at(FORTY_TWO_WEEKS_GESTATION), Ok(TableId::Who2006Infant));
        assert_eq!(at(1.999), Ok(TableId::Who2006Infant));
        assert_eq!(at(2.0), Ok(TableId::Who2006Child));
        assert_eq!(at(3.999), Ok(TableId::Who2006Child));
        assert_eq!(at(4.0), Ok(TableId::Uk90Child));
        assert_eq!(at(20.0), Ok(TableId::Uk90Child));
        assert_eq!(at(20.001), Err(UkWhoAbsence::AboveTwentyYears));
        assert_eq!(at(-0.4), Err(UkWhoAbsence::BelowTwentyThreeWeeks));
    }

    #[test]
    fn youngest_at_boundary_only_changes_the_exact_joins() {
        let young = |age| table_for_age(age, true);
        assert_eq!(young(FORTY_TWO_WEEKS_GESTATION), Ok(TableId::Uk90Preterm));
        assert_eq!(young(2.0), Ok(TableId::Who2006Infant));
        assert_eq!(young(4.0), Ok(TableId::Who2006Child));
        assert_eq!(young(1.0), Ok(TableId::Who2006Infant));
    }

    #[test]
    fn absence_rules() {
        use MeasurementMethod::{Bmi, Height, Ofc, Weight};
        assert_eq!(
            data_absent(-0.3, Height, Sex::Male),
            Some(UkWhoAbsence::LengthBelowTwentyFiveWeeks)
        );
        assert_eq!(data_absent(-0.3, Weight, Sex::Male), None);
        assert_eq!(
            data_absent(0.0, Bmi, Sex::Female),
            Some(UkWhoAbsence::BmiBelowTwoWeeks)
        );
        assert_eq!(
            data_absent(17.5, Ofc, Sex::Female),
            Some(UkWhoAbsence::OfcAboveSeventeenFemale)
        );
        assert_eq!(data_absent(17.5, Ofc, Sex::Male), None);
        assert_eq!(
            data_absent(18.5, Ofc, Sex::Male),
            Some(UkWhoAbsence::OfcAboveEighteenMale)
        );
    }
}
