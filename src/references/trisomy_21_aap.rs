// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-FileCopyrightText: Royal College of Paediatrics and Child Health
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The AAP Trisomy 21 reference, ported from `rcpchgrowth.trisomy_21_aap`:
//! an infant table to 3 years and a child table from 2 to 20 years.
//!
//! Upstream prefixes each absence message with "Reference data absent: ",
//! and its routes pass no bone age to the calculation.

use super::{
    Absence, Api, ChartRules, Correction, Definition, LineSds, SdsMethod, SdsRules, Segment,
    cole_nine_centiles,
};
use crate::chart::ages;
use crate::data::TableId;
use crate::table::Interpolation;
use crate::vocabulary::{GrowthReference, MeasurementMethod, Sex};

/// The first age in the AAP infant table for height and head circumference.
pub const AAP_INFANT_FIRST_AGE: f64 = 0.083_333_333;

/// AAP Trisomy 21.
pub static DEFINITION: Definition = Definition {
    reference: GrowthReference::Trisomy21Aap,
    label: "Trisomy 21 (AAP)",
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
        drops_overflowing_lines: false,
    },
    adult_age: 20.0,
    api: Api {
        prefix: "trisomy-21-aap",
        chart_name: "trisomy-21-aap",
        fictional_child_name: "trisomy-21-aap",
        only: None,
        bone_age: false,
    },
};

const fn absent(message: &'static str) -> Absence {
    Absence {
        reference: GrowthReference::Trisomy21Aap,
        message,
    }
}

/// Below term.
pub const BELOW_TERM: Absence = absent(
    "Reference data absent: No Trisomy 21 (AAP) reference data exists below 40 weeks gestation",
);
/// Height or head circumference below 1 month.
pub const BELOW_ONE_MONTH: Absence = absent(
    "Reference data absent: No Trisomy 21 (AAP) reference data exists below 1 month of age for height or head circumference.",
);
/// BMI below 2 years.
pub const BMI_BELOW_TWO_YEARS: Absence = absent(
    "Reference data absent: No Trisomy 21 (AAP) reference data exists below 2 years of age for BMI.",
);
/// Above 20 years.
pub const ABOVE_TWENTY_YEARS: Absence =
    absent("Reference data absent: Trisomy 21 reference data does not exist over the age of 20y.");

fn select(
    age: f64,
    method: MeasurementMethod,
    _sex: Sex,
    youngest_at_boundary: bool,
) -> Result<TableId, Absence> {
    trisomy_21_aap(age, method, youngest_at_boundary)
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
) -> Result<TableId, Absence> {
    use MeasurementMethod::{Bmi, Height, Ofc, Weight};
    if age < 0.0 {
        return Err(BELOW_TERM);
    }
    if age < AAP_INFANT_FIRST_AGE && matches!(method, Height | Ofc) {
        return Err(BELOW_ONE_MONTH);
    }
    if age < 2.0 && method == Bmi {
        return Err(BMI_BELOW_TWO_YEARS);
    }
    if age > 20.0 {
        return Err(ABOVE_TWENTY_YEARS);
    }
    if age <= 3.0 && matches!(method, Height | Weight | Ofc) {
        if method == Ofc && age == 3.0 && !youngest_at_boundary {
            return Ok(TableId::Trisomy21AapChild);
        }
        return Ok(TableId::Trisomy21AapInfant);
    }
    Ok(TableId::Trisomy21AapChild)
}

#[allow(clippy::float_cmp, reason = "upstream compares these ages exactly")]
const SEGMENTS: [Segment; 2] = [
    Segment {
        name: "trisomy_21_aap_infant",
        ages: |_| &ages::TRISOMY_21_AAP_INFANT_AGES,
        youngest_at: |age| age == 3.0,
    },
    Segment {
        name: "trisomy_21_aap_child",
        ages: |_| &ages::TRISOMY_21_AAP_CHILD_AGES,
        youngest_at: |_| false,
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use MeasurementMethod::{Bmi, Height, Ofc, Weight};

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
        assert_eq!(trisomy_21_aap(0.05, Ofc, false), Err(BELOW_ONE_MONTH));
    }
}
