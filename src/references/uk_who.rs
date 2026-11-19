// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-FileCopyrightText: Royal College of Paediatrics and Child Health
// SPDX-License-Identifier: AGPL-3.0-or-later

//! UK-WHO, ported from `rcpchgrowth.uk_who`.
//!
//! UK-WHO stitches together three sources:
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
//! calculation never does. Every gestation is corrected for, and the Cole
//! nine-centile collection is drawn at SDS rounded to two-thirds.

use super::{
    Absence, Api, ChartRules, Correction, Definition, LineSds, SdsMethod, SdsRules, Segment,
    cole_nine_centiles,
};
use crate::chart::ages;
use crate::data::TableId;
use crate::table::Interpolation;
use crate::vocabulary::{GrowthReference, MeasurementMethod, Sex};

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

/// UK-WHO.
pub static DEFINITION: Definition = Definition {
    reference: GrowthReference::UkWho,
    label: "UK-WHO",
    select,
    sds: SdsRules {
        interpolation: Interpolation::CubicWherePossible,
        method: SdsMethod::Lms,
        clamp_at_two_years: true,
    },
    correction: Correction::Always,
    before_term: None,
    centile_format: cole_nine_centiles,
    chart: ChartRules {
        segments: &SEGMENTS,
        cole_rounding: true,
        line_sds: LineSds::Hundredths,
        drops_overflowing_lines: false,
    },
    adult_age: 20.0,
    api: Api {
        prefix: "uk-who",
        chart_name: "UK-WHO",
        fictional_child_name: "UK-WHO",
        only: None,
        bone_age: true,
    },
};

const fn absent(message: &'static str) -> Absence {
    Absence {
        reference: GrowthReference::UkWho,
        message,
    }
}

/// Below 23 weeks gestation.
pub const BELOW_TWENTY_THREE_WEEKS: Absence =
    absent("There is no UK90 reference data below 23 weeks gestation");
/// Above 20 years.
pub const ABOVE_TWENTY_YEARS: Absence =
    absent("There is no UK90 reference data above the age of 20 years.");
/// Length below 25 weeks gestation.
pub const LENGTH_BELOW_TWENTY_FIVE_WEEKS: Absence =
    absent("UK-WHO length data does not exist in infants below 25 weeks gestation.");
/// BMI below 2 weeks of age.
pub const BMI_BELOW_TWO_WEEKS: Absence =
    absent("UK-WHO BMI data does not exist below 2 weeks of age.");
/// Head circumference above 18 years in boys.
pub const OFC_ABOVE_EIGHTEEN_MALE: Absence =
    absent("UK-WHO head circumference data does not exist in boys over 18 y of age.");
/// Head circumference above 17 years in girls.
pub const OFC_ABOVE_SEVENTEEN_FEMALE: Absence =
    absent("UK-WHO head circumference data does not exist in girls over 17 y of age.");

/// The table for a measurement at an age: the source table first, then the
/// absence rules, as upstream checks them.
fn select(
    age: f64,
    method: MeasurementMethod,
    sex: Sex,
    youngest_at_boundary: bool,
) -> Result<TableId, Absence> {
    let table = table_for_age(age, youngest_at_boundary)?;
    match data_absent(age, method, sex) {
        Some(absence) => Err(absence),
        None => Ok(table),
    }
}

/// The source table for `age` (the upstream `uk_who_reference`).
///
/// # Errors
///
/// Below 23 weeks gestation or above 20 years.
#[allow(
    clippy::float_cmp,
    reason = "upstream compares the boundary ages exactly"
)]
pub fn table_for_age(age: f64, youngest_at_boundary: bool) -> Result<TableId, Absence> {
    if age < TWENTY_THREE_WEEKS_GESTATION {
        Err(BELOW_TWENTY_THREE_WEEKS)
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
        Err(ABOVE_TWENTY_YEARS)
    }
}

/// Whether UK-WHO lacks data for this measurement at this age (the upstream
/// `reference_data_absent`). Checked after [`table_for_age`], as upstream.
#[must_use]
pub fn data_absent(age: f64, method: MeasurementMethod, sex: Sex) -> Option<Absence> {
    // The age bounds repeat `table_for_age`, so they are unreachable in
    // sequence but kept so this function is correct on its own.
    if age < TWENTY_THREE_WEEKS_GESTATION {
        return Some(BELOW_TWENTY_THREE_WEEKS);
    }
    if age > UK90_UPPER_THRESHOLD {
        return Some(ABOVE_TWENTY_YEARS);
    }
    match method {
        MeasurementMethod::Height if age < TWENTY_FIVE_WEEKS_GESTATION => {
            Some(LENGTH_BELOW_TWENTY_FIVE_WEEKS)
        }
        MeasurementMethod::Bmi if age < FORTY_TWO_WEEKS_GESTATION => Some(BMI_BELOW_TWO_WEEKS),
        MeasurementMethod::Ofc => match sex {
            Sex::Male if age > OFC_UPPER_MALE => Some(OFC_ABOVE_EIGHTEEN_MALE),
            Sex::Female if age > OFC_UPPER_FEMALE => Some(OFC_ABOVE_SEVENTEEN_FEMALE),
            _ => None,
        },
        _ => None,
    }
}

#[allow(clippy::float_cmp, reason = "upstream compares these ages exactly")]
const SEGMENTS: [Segment; 4] = [
    Segment {
        name: "uk90_preterm",
        ages: |_| &ages::UK_90_PRETERM_AGES,
        youngest_at: |age| age == 0.038_329_911,
    },
    Segment {
        name: "uk_who_infant",
        ages: |_| &ages::WHO_2006_UNDER_TWOS_AGES,
        youngest_at: |age| age == 2.0,
    },
    Segment {
        name: "uk_who_child",
        ages: |_| &ages::UK_WHO_2006_OVER_TWOS_AGES,
        youngest_at: |age| age == 4.0,
    },
    Segment {
        name: "uk90_child",
        ages: |_| &ages::UK90_AGES,
        youngest_at: |_| false,
    },
];

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
        assert_eq!(at(20.001), Err(ABOVE_TWENTY_YEARS));
        assert_eq!(at(-0.4), Err(BELOW_TWENTY_THREE_WEEKS));
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
            Some(LENGTH_BELOW_TWENTY_FIVE_WEEKS)
        );
        assert_eq!(data_absent(-0.3, Weight, Sex::Male), None);
        assert_eq!(
            data_absent(0.0, Bmi, Sex::Female),
            Some(BMI_BELOW_TWO_WEEKS)
        );
        assert_eq!(
            data_absent(17.5, Ofc, Sex::Female),
            Some(OFC_ABOVE_SEVENTEEN_FEMALE)
        );
        assert_eq!(data_absent(17.5, Ofc, Sex::Male), None);
        assert_eq!(
            data_absent(18.5, Ofc, Sex::Male),
            Some(OFC_ABOVE_EIGHTEEN_MALE)
        );
    }
}
