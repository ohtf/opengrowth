// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-FileCopyrightText: Royal College of Paediatrics and Child Health
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The UK Turner syndrome reference, ported from `rcpchgrowth.turner`: girls'
//! height from 1 to 20 years, in one table.
//!
//! Upstream's routes for it live under `/turner` and refuse any other sex or
//! measurement before calculating, each with its own message, and its chart
//! builder leaves out a line whose SDS overflows rather than failing.

use super::{
    Absence, Api, ChartRules, Correction, Definition, LineSds, Only, SdsMethod, SdsRules, Segment,
    cole_nine_centiles,
};
use crate::chart::ages;
use crate::data::TableId;
use crate::table::Interpolation;
use crate::vocabulary::{GrowthReference, MeasurementMethod, Sex};

/// Turner syndrome.
pub static DEFINITION: Definition = Definition {
    reference: GrowthReference::TurnersSyndrome,
    label: "Turner syndrome",
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
        line_sds: LineSds::Hundredths,
        drops_overflowing_lines: true,
    },
    adult_age: 20.0,
    api: Api {
        prefix: "turner",
        chart_name: "Turner's syndrome",
        fictional_child_name: "Turner",
        only: Some(Only {
            sex: Sex::Female,
            method: MeasurementMethod::Height,
            calculation_sex: "Turner reference data only exists in girls.",
            calculation_method: "Turner reference data only exists for height",
            chart: "Turner data only exists for height in girls.",
            fictional_child_sex: "Turner's Syndrome data only exists for girls.",
            fictional_child_method: "Turner's Syndrome data only exists for height.",
        }),
        bone_age: true,
    },
};

const fn absent(message: &'static str) -> Absence {
    Absence {
        reference: GrowthReference::TurnersSyndrome,
        message,
    }
}

/// Below 1 year.
pub const BELOW_ONE_YEAR: Absence = absent("There is no reference data below 1 year.");
/// Above 20 years.
pub const ABOVE_TWENTY_YEARS: Absence = absent("There is no reference data above 20 years.");
/// Weight.
pub const WEIGHT: Absence = absent("There is no reference data for weight.");
/// BMI.
pub const BMI: Absence = absent("There is no reference data for BMI (body mass index).");
/// Head circumference.
pub const OFC: Absence = absent("There is no reference data for head circumference.");
/// A boy.
pub const MALE: Absence = absent("Turner's syndrome only affects girls and women.");

fn select(
    age: f64,
    method: MeasurementMethod,
    sex: Sex,
    _youngest_at_boundary: bool,
) -> Result<TableId, Absence> {
    turner(age, method, sex)
}

/// The Turner table for a request (upstream `reference_data_absent` and
/// `turner_lms_array_for_measurement_and_sex`): girls' height, 1 to 20 years.
///
/// # Errors
///
/// Where there is no Turner data, with upstream's message.
pub fn turner(age: f64, method: MeasurementMethod, sex: Sex) -> Result<TableId, Absence> {
    if age < 1.0 {
        Err(BELOW_ONE_YEAR)
    } else if age > 20.0 {
        Err(ABOVE_TWENTY_YEARS)
    } else if method == MeasurementMethod::Weight {
        Err(WEIGHT)
    } else if method == MeasurementMethod::Bmi {
        Err(BMI)
    } else if method == MeasurementMethod::Ofc {
        Err(OFC)
    } else if sex == Sex::Male {
        Err(MALE)
    } else {
        Ok(TableId::Turner)
    }
}

const SEGMENTS: [Segment; 1] = [Segment {
    name: "turners-syndrome",
    ages: |_| &ages::TURNER_AGES,
    youngest_at: |_| false,
}];

#[cfg(test)]
mod tests {
    use super::*;
    use MeasurementMethod::{Bmi, Height};

    #[test]
    fn turner_is_girls_height_from_one_to_twenty() {
        assert_eq!(turner(5.0, Height, Sex::Female), Ok(TableId::Turner));
        assert_eq!(turner(0.5, Height, Sex::Female), Err(BELOW_ONE_YEAR));
        assert_eq!(turner(5.0, Bmi, Sex::Male), Err(BMI));
        assert_eq!(turner(5.0, Height, Sex::Male), Err(MALE));
    }
}
