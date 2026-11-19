// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-FileCopyrightText: Royal College of Paediatrics and Child Health
// SPDX-License-Identifier: AGPL-3.0-or-later

//! CDC, ported from `rcpchgrowth.cdc`.
//!
//! CDC joins the earlier WHO 2006 infant table (height and weight to 2
//! years), the CDC infant table (head circumference to 3 years), and CDC
//! 2000 from 2 to 20 years, whose BMI carries the extended-BMI sigma.
//! Upstream's preterm segment, Fenton, has no data, so there is nothing
//! below term, and a preterm baby before term gets no corrected SDS.
//!
//! Upstream works out whether data is absent first but reports it last: the
//! table choice can fail first, so its messages win below 22 weeks and above
//! 20 years. [`lookup`] keeps that order.
//!
//! BMI above the 95th centile uses the CDC extended BMI-for-age method.
//! Ages follow the term rule, as for WHO. Centiles are described with the
//! three-percent collection, and BMI with the 85th-centile one.

use super::{
    Absence, Api, ChartRules, Correction, Definition, LineSds, SdsMethod, SdsRules, Segment,
};
use crate::advice::CentileFormat;
use crate::chart::ages;
use crate::data::TableId;
use crate::table::Interpolation;
use crate::vocabulary::{GrowthReference, MeasurementMethod, Sex};

/// 22 weeks gestation, as decimal age: the start of upstream's Fenton segment.
pub const FENTON_LOWER_THRESHOLD: f64 = ((22 * 7) - (40 * 7)) as f64 / 365.25;
/// CDC data end at 20 years.
pub const CDC_UPPER_THRESHOLD: f64 = 20.0;

/// CDC.
pub static DEFINITION: Definition = Definition {
    reference: GrowthReference::Cdc,
    label: "CDC",
    select,
    sds: SdsRules {
        interpolation: Interpolation::CubicWherePossible,
        method: SdsMethod::CdcExtendedBmi,
        clamp_at_two_years: false,
    },
    correction: Correction::TermRule,
    before_term: Some(
        "This baby is born premature. CDC data is not available for preterm infants.",
    ),
    centile_format,
    chart: ChartRules {
        segments: &SEGMENTS,
        cole_rounding: false,
        line_sds: LineSds::Hundredths,
        drops_overflowing_lines: false,
    },
    adult_age: 20.0,
    api: Api {
        prefix: "cdc",
        chart_name: "CDC",
        fictional_child_name: "CDC",
        only: None,
        bone_age: true,
    },
};

const fn absent(message: &'static str) -> Absence {
    Absence {
        reference: GrowthReference::Cdc,
        message,
    }
}

/// Below 22 weeks gestation.
pub const BELOW_TWENTY_TWO_WEEKS: Absence =
    absent("There is no reference data for ages below 22 weeks gestation.");
/// Between 22 weeks and term: upstream's Fenton segment, which is empty.
pub const BELOW_TERM: Absence = absent("CDC data does not exist below 40 weeks.");
/// Above 20 years.
pub const ABOVE_TWENTY_YEARS: Absence =
    absent("There is no CDC reference data above the age of 20 years.");
/// Head circumference above 3 years.
pub const OFC_ABOVE_THREE_YEARS: Absence =
    absent("CDC data does not exist for head circumference above 3 years.");
/// BMI below 2 years.
pub const BMI_BELOW_TWO_YEARS: Absence = absent("CDC data does not exist for BMI below 2 years.");

fn select(
    age: f64,
    method: MeasurementMethod,
    _sex: Sex,
    youngest_at_boundary: bool,
) -> Result<TableId, Absence> {
    lookup(age, method, youngest_at_boundary)
}

/// The table for this measurement at this age (the upstream
/// `cdc_lms_array_for_measurement_and_sex`). `youngest_at_boundary` chooses
/// the infant table at exactly 2 years.
///
/// # Errors
///
/// Where there is no CDC data, with upstream's message.
#[allow(
    clippy::float_cmp,
    reason = "upstream compares the boundary age exactly"
)]
pub fn lookup(
    age: f64,
    method: MeasurementMethod,
    youngest_at_boundary: bool,
) -> Result<TableId, Absence> {
    let absent = data_absent(age, method);
    // The upstream `cdc_reference`; `None` is the empty Fenton segment.
    let table = if age < FENTON_LOWER_THRESHOLD {
        return Err(BELOW_TWENTY_TWO_WEEKS);
    } else if age < 0.0 {
        None
    } else if method == MeasurementMethod::Ofc && age <= 3.0 {
        Some(TableId::CdcInfant)
    } else if age < 2.0 || (age == 2.0 && youngest_at_boundary) {
        Some(TableId::Who2006InfantPre2025)
    } else if age <= CDC_UPPER_THRESHOLD {
        Some(TableId::CdcChild)
    } else {
        return Err(ABOVE_TWENTY_YEARS);
    };
    match (absent, table) {
        (Some(absence), _) => Err(absence),
        (None, Some(table)) => Ok(table),
        // Unreachable: an age below 0 is always absent.
        (None, None) => Err(BELOW_TERM),
    }
}

/// Whether CDC lacks data for this measurement at this age (the upstream
/// `reference_data_absent`). Above 20 years the table choice fails first,
/// with its own message, so that case is not repeated here.
fn data_absent(age: f64, method: MeasurementMethod) -> Option<Absence> {
    if age < 0.0 {
        Some(BELOW_TERM)
    } else if method == MeasurementMethod::Ofc && age > 3.0 {
        Some(OFC_ABOVE_THREE_YEARS)
    } else if method == MeasurementMethod::Bmi && age < 2.0 {
        Some(BMI_BELOW_TWO_YEARS)
    } else {
        None
    }
}

/// The three-percent collection, and the 85th-centile one for BMI.
const fn centile_format(method: MeasurementMethod) -> CentileFormat {
    match method {
        MeasurementMethod::Bmi => CentileFormat::EightyFivePercentCentiles,
        _ => CentileFormat::ThreePercentCentiles,
    }
}

#[allow(clippy::float_cmp, reason = "upstream compares these ages exactly")]
const SEGMENTS: [Segment; 3] = [
    // Upstream's Fenton preterm segment has no ages, so its lines are empty.
    Segment {
        name: "fenton",
        ages: |_| &[],
        youngest_at: |_| false,
    },
    Segment {
        name: "cdc_infant",
        ages: |method| match method {
            MeasurementMethod::Ofc => &ages::CDC_TO_THREE_AGE,
            MeasurementMethod::Height | MeasurementMethod::Weight => {
                &ages::WHO_2006_UNDER_TWOS_AGES
            }
            MeasurementMethod::Bmi => &ages::CDC_TO_TWO_AGE,
        },
        youngest_at: |age| age == 0.038_329_911 || age == 2.0,
    },
    Segment {
        name: "cdc_child",
        ages: |_| &ages::CDC_TWO_TWENTY,
        youngest_at: |_| false,
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use MeasurementMethod::{Bmi, Height, Ofc, Weight};

    #[test]
    fn tables_and_absences_follow_upstream_order() {
        assert_eq!(lookup(-0.4, Height, false), Err(BELOW_TWENTY_TWO_WEEKS));
        assert_eq!(lookup(-0.1, Weight, false), Err(BELOW_TERM));
        assert_eq!(lookup(0.5, Ofc, false), Ok(TableId::CdcInfant));
        assert_eq!(lookup(3.0, Ofc, false), Ok(TableId::CdcInfant));
        assert_eq!(lookup(3.1, Ofc, false), Err(OFC_ABOVE_THREE_YEARS));
        assert_eq!(
            lookup(1.0, Height, false),
            Ok(TableId::Who2006InfantPre2025)
        );
        assert_eq!(lookup(2.0, Height, true), Ok(TableId::Who2006InfantPre2025));
        assert_eq!(lookup(2.0, Height, false), Ok(TableId::CdcChild));
        assert_eq!(lookup(1.0, Bmi, false), Err(BMI_BELOW_TWO_YEARS));
        assert_eq!(lookup(20.5, Weight, false), Err(ABOVE_TWENTY_YEARS));
    }
}
