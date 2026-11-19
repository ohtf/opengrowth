// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-FileCopyrightText: Royal College of Paediatrics and Child Health
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The UK Trisomy 21 reference, ported from `rcpchgrowth.trisomy_21`: birth
//! to 20 years, in one table.
//!
//! Upstream's chart builder for it rounds a line's SDS itself, as its float
//! kind rounds, rather than rounding 100 times it to an integer.

use super::{
    Absence, Api, ChartRules, Correction, Definition, LineSds, SdsMethod, SdsRules, Segment,
    cole_nine_centiles,
};
use crate::chart::ages;
use crate::data::TableId;
use crate::table::Interpolation;
use crate::vocabulary::{GrowthReference, MeasurementMethod, Sex};

/// UK Trisomy 21.
pub static DEFINITION: Definition = Definition {
    reference: GrowthReference::Trisomy21,
    label: "Trisomy 21 (UK)",
    select,
    sds: SdsRules {
        interpolation: Interpolation::CubicWherePossible,
        method: SdsMethod::Lms,
        clamp_at_two_years: false,
    },
    correction: Correction::Always,
    before_term: None,
    centile_format: cole_nine_centiles,
    chart: ChartRules {
        segments: &SEGMENTS,
        cole_rounding: false,
        line_sds: LineSds::Rounded,
        drops_overflowing_lines: false,
    },
    adult_age: 20.0,
    api: Api {
        prefix: "trisomy-21",
        chart_name: "Trisomy 21",
        fictional_child_name: "Trisomy 21",
        only: None,
        bone_age: true,
    },
};

const fn absent(message: &'static str) -> Absence {
    Absence {
        reference: GrowthReference::Trisomy21,
        message,
    }
}

/// Below term.
pub const BELOW_TERM: Absence = absent("No reference data exists below 40 weeks gestation");
/// Above 20 years.
pub const ABOVE_TWENTY_YEARS: Absence =
    absent("Trisomy 21 reference data does not exist over the age of 20y.");
/// BMI above 18.82 years. The table runs to 18.92, so two rows per sex are
/// unreachable (`spec/standard.md` section 6.2).
pub const BMI_ABOVE_LIMIT: Absence = absent("Trisomy BMI reference data does not exist > 18.82 y.");
/// Head circumference above 18 years.
pub const OFC_ABOVE_EIGHTEEN_YEARS: Absence =
    absent("Trisomy head circumference reference data does not exist > 18 y");

fn select(
    age: f64,
    method: MeasurementMethod,
    _sex: Sex,
    _youngest_at_boundary: bool,
) -> Result<TableId, Absence> {
    trisomy_21(age, method)
}

/// The UK Trisomy 21 table for a request (upstream `trisomy_21`).
///
/// # Errors
///
/// Where there is no Trisomy 21 data, with upstream's message.
pub fn trisomy_21(age: f64, method: MeasurementMethod) -> Result<TableId, Absence> {
    if age < 0.0 {
        Err(BELOW_TERM)
    } else if age > 20.0 {
        Err(ABOVE_TWENTY_YEARS)
    } else if method == MeasurementMethod::Bmi && age > 18.82 {
        Err(BMI_ABOVE_LIMIT)
    } else if method == MeasurementMethod::Ofc && age > 18.0 {
        Err(OFC_ABOVE_EIGHTEEN_YEARS)
    } else {
        Ok(TableId::Trisomy21)
    }
}

const SEGMENTS: [Segment; 1] = [Segment {
    name: "trisomy-21",
    ages: |_| &ages::TRISOMY_21_AGES,
    youngest_at: |_| false,
}];

#[cfg(test)]
mod tests {
    use super::*;
    use MeasurementMethod::{Bmi, Ofc, Weight};

    #[test]
    fn trisomy_21_limits_follow_upstream() {
        assert_eq!(trisomy_21(18.9, Bmi), Err(BMI_ABOVE_LIMIT));
        assert_eq!(trisomy_21(18.5, Ofc), Err(OFC_ABOVE_EIGHTEEN_YEARS));
        assert_eq!(trisomy_21(17.0, Weight), Ok(TableId::Trisomy21));
    }
}
