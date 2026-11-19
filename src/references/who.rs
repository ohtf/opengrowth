// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-FileCopyrightText: Royal College of Paediatrics and Child Health
// SPDX-License-Identifier: AGPL-3.0-or-later

//! WHO, ported from `rcpchgrowth.who`.
//!
//! WHO joins three tables: the WHO 2006 growth standard from birth to 2
//! years and from 2 years to 1856 days (5.08 years), both daily, then the
//! WHO 2007 growth reference, monthly, to 19 years. Unlike UK-WHO there is
//! no preterm table: WHO has no data below term. Absence is checked before
//! the table is chosen, as upstream.
//!
//! SDS lookups interpolate linearly, and beyond 3 SD the restricted LMS
//! method applies. Ages follow the term rule: 37 to 42 weeks count as term,
//! and correction stops at 2 years.

use super::{
    Absence, Api, ChartRules, Correction, Definition, LineSds, SdsMethod, SdsRules, Segment,
    cole_nine_centiles,
};
use crate::chart::ages;
use crate::data::TableId;
use crate::table::Interpolation;
use crate::vocabulary::{GrowthReference, MeasurementMethod, Sex};

/// The upper end of the WHO 2006 child table: 1856 days.
pub const WHO_2006_UPPER_THRESHOLD: f64 = 1856.0 / 365.25;
/// The WHO 2006 infant table ends, and the child table starts, at 2 years.
pub const WHO_CHILD_LOWER_THRESHOLD: f64 = 2.0;
/// The WHO 2007 reference ends at 19 years.
pub const WHO_2007_UPPER_THRESHOLD: f64 = 19.0;
/// WHO weight data stop at 10 years.
pub const WEIGHT_UPPER: f64 = 10.0;

/// WHO.
pub static DEFINITION: Definition = Definition {
    reference: GrowthReference::Who,
    label: "WHO",
    select,
    sds: SdsRules {
        interpolation: Interpolation::LinearOnly,
        method: SdsMethod::WhoRestricted,
        clamp_at_two_years: true,
    },
    correction: Correction::TermRule,
    before_term: None,
    centile_format: cole_nine_centiles,
    chart: ChartRules {
        segments: &SEGMENTS,
        cole_rounding: false,
        line_sds: LineSds::Hundredths,
        drops_overflowing_lines: false,
    },
    // WHO's reference ends at 19 years.
    adult_age: 19.0,
    api: Api {
        prefix: "who",
        chart_name: "WHO",
        fictional_child_name: "WHO",
        only: None,
        bone_age: true,
    },
};

const fn absent(message: &'static str) -> Absence {
    Absence {
        reference: GrowthReference::Who,
        message,
    }
}

/// Below term (corrected age 0).
pub const BELOW_TERM: Absence = absent("WHO data does not exist below term.");
/// Above 19 years.
pub const ABOVE_NINETEEN_YEARS: Absence = absent("WHO data does not exist above 19 years.");
/// Weight above 10 years.
pub const WEIGHT_ABOVE_TEN_YEARS: Absence =
    absent("WHO weight data does not exist in children over 10 y of age.");
/// Head circumference above the WHO 2006 standard.
pub const OFC_ABOVE_FIVE_YEARS: Absence =
    absent("WHO head circumference data does not exist in children over 5 y of age.");
/// No table above 19 years (upstream `who_reference`, which the absence
/// rules make unreachable in sequence).
pub const NO_TABLE_ABOVE_NINETEEN_YEARS: Absence =
    absent("There are no WHO reference data above the age of 19 years.");

/// The table for a measurement at an age: the absence rules first, then
/// the source table, as upstream checks them.
fn select(
    age: f64,
    method: MeasurementMethod,
    _sex: Sex,
    youngest_at_boundary: bool,
) -> Result<TableId, Absence> {
    match data_absent(age, method) {
        Some(absence) => Err(absence),
        None => table_for_age(age, youngest_at_boundary),
    }
}

/// Whether WHO lacks data for this measurement at this age (the upstream
/// `reference_data_absent`). Checked before [`table_for_age`], as upstream.
#[must_use]
pub fn data_absent(age: f64, method: MeasurementMethod) -> Option<Absence> {
    if age < 0.0 {
        Some(BELOW_TERM)
    } else if age > 19.0 {
        Some(ABOVE_NINETEEN_YEARS)
    } else if method == MeasurementMethod::Weight && age > WEIGHT_UPPER {
        Some(WEIGHT_ABOVE_TEN_YEARS)
    } else if method == MeasurementMethod::Ofc && age > WHO_2006_UPPER_THRESHOLD {
        Some(OFC_ABOVE_FIVE_YEARS)
    } else {
        None
    }
}

/// The source table for `age` (the upstream `who_reference`).
/// `youngest_at_boundary` chooses the infant table at exactly 2 years.
///
/// # Errors
///
/// Above 19 years.
#[allow(
    clippy::float_cmp,
    reason = "upstream compares the boundary age exactly"
)]
pub fn table_for_age(age: f64, youngest_at_boundary: bool) -> Result<TableId, Absence> {
    if age <= WHO_2006_UPPER_THRESHOLD {
        if (age == 2.0 && youngest_at_boundary) || age < WHO_CHILD_LOWER_THRESHOLD {
            Ok(TableId::Who2006Infant)
        } else {
            Ok(TableId::Who2006Child)
        }
    } else if age <= WHO_2007_UPPER_THRESHOLD {
        Ok(TableId::Who2007Child)
    } else {
        Err(NO_TABLE_ABOVE_NINETEEN_YEARS)
    }
}

#[allow(clippy::float_cmp, reason = "upstream compares these ages exactly")]
const SEGMENTS: [Segment; 3] = [
    Segment {
        name: "who_2006_infant",
        ages: |_| &ages::WHO_2006_UNDER_TWOS_AGES,
        youngest_at: |age| age == 0.038_329_911 || age == 2.0,
    },
    Segment {
        name: "who_2006_child",
        ages: |_| &ages::WHO_2006_OVER_TWOS_AGES,
        youngest_at: |age| age == 5.0,
    },
    Segment {
        name: "who_2007_child",
        ages: |_| &ages::WHO_2007_AGES,
        youngest_at: |_| false,
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tables_switch_at_two_years_and_1856_days() {
        assert_eq!(table_for_age(1.999, false), Ok(TableId::Who2006Infant));
        assert_eq!(table_for_age(2.0, false), Ok(TableId::Who2006Child));
        assert_eq!(table_for_age(2.0, true), Ok(TableId::Who2006Infant));
        assert_eq!(
            table_for_age(WHO_2006_UPPER_THRESHOLD, false),
            Ok(TableId::Who2006Child)
        );
        assert_eq!(
            table_for_age(1857.0 / 365.25, false),
            Ok(TableId::Who2007Child)
        );
        assert_eq!(table_for_age(19.0, false), Ok(TableId::Who2007Child));
        assert!(table_for_age(19.1, false).is_err());
    }

    #[test]
    fn absence_follows_upstream_order() {
        assert_eq!(
            data_absent(-0.01, MeasurementMethod::Height),
            Some(BELOW_TERM)
        );
        assert_eq!(
            data_absent(19.5, MeasurementMethod::Weight),
            Some(ABOVE_NINETEEN_YEARS)
        );
        assert_eq!(
            data_absent(10.5, MeasurementMethod::Weight),
            Some(WEIGHT_ABOVE_TEN_YEARS)
        );
        assert_eq!(
            data_absent(5.1, MeasurementMethod::Ofc),
            Some(OFC_ABOVE_FIVE_YEARS)
        );
        assert_eq!(data_absent(5.1, MeasurementMethod::Bmi), None);
    }
}
