// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-FileCopyrightText: Royal College of Paediatrics and Child Health
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The CDC reference: which table serves an age, and where there is none.
//!
//! Ported from `rcpchgrowth.cdc`. CDC joins the earlier WHO 2006 infant table
//! (height and weight to 2 years), the CDC infant table (head circumference
//! to 3 years), and CDC 2000 from 2 to 20 years, whose BMI carries the
//! extended-BMI sigma. Upstream's preterm segment, Fenton, has no data, so
//! there is nothing below term.
//!
//! Upstream works out whether data is absent first but reports it last: the
//! table choice can fail first, so its messages win below 22 weeks and above
//! 20 years. [`lookup`] keeps that order.

use crate::data::TableId;
use crate::vocabulary::MeasurementMethod;

/// 22 weeks gestation, as decimal age: the start of upstream's Fenton segment.
pub const FENTON_LOWER_THRESHOLD: f64 = ((22 * 7) - (40 * 7)) as f64 / 365.25;
/// CDC data end at 20 years.
pub const CDC_UPPER_THRESHOLD: f64 = 20.0;

/// No CDC data exists for this request. Each message is upstream's
/// `LookupError` text, which reaches API responses unchanged.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum CdcAbsence {
    /// Below 22 weeks gestation.
    #[error("There is no reference data for ages below 22 weeks gestation.")]
    BelowTwentyTwoWeeks,
    /// Between 22 weeks and term: upstream's Fenton segment, which is empty.
    #[error("CDC data does not exist below 40 weeks.")]
    BelowTerm,
    /// Above 20 years.
    #[error("There is no CDC reference data above the age of 20 years.")]
    AboveTwentyYears,
    /// Head circumference above 3 years.
    #[error("CDC data does not exist for head circumference above 3 years.")]
    OfcAboveThreeYears,
    /// BMI below 2 years.
    #[error("CDC data does not exist for BMI below 2 years.")]
    BmiBelowTwoYears,
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
) -> Result<TableId, CdcAbsence> {
    let absent = data_absent(age, method);
    // The upstream `cdc_reference`; `None` is the empty Fenton segment.
    let table = if age < FENTON_LOWER_THRESHOLD {
        return Err(CdcAbsence::BelowTwentyTwoWeeks);
    } else if age < 0.0 {
        None
    } else if method == MeasurementMethod::Ofc && age <= 3.0 {
        Some(TableId::CdcInfant)
    } else if age < 2.0 || (age == 2.0 && youngest_at_boundary) {
        Some(TableId::Who2006InfantPre2025)
    } else if age <= CDC_UPPER_THRESHOLD {
        Some(TableId::CdcChild)
    } else {
        return Err(CdcAbsence::AboveTwentyYears);
    };
    match (absent, table) {
        (Some(absence), _) => Err(absence),
        (None, Some(table)) => Ok(table),
        // Unreachable: an age below 0 is always absent.
        (None, None) => Err(CdcAbsence::BelowTerm),
    }
}

/// Whether CDC lacks data for this measurement at this age (the upstream
/// `reference_data_absent`). Above 20 years the table choice fails first,
/// with its own message, so that case is not repeated here.
fn data_absent(age: f64, method: MeasurementMethod) -> Option<CdcAbsence> {
    if age < 0.0 {
        Some(CdcAbsence::BelowTerm)
    } else if method == MeasurementMethod::Ofc && age > 3.0 {
        Some(CdcAbsence::OfcAboveThreeYears)
    } else if method == MeasurementMethod::Bmi && age < 2.0 {
        Some(CdcAbsence::BmiBelowTwoYears)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use MeasurementMethod::{Bmi, Height, Ofc, Weight};

    #[test]
    fn tables_and_absences_follow_upstream_order() {
        assert_eq!(
            lookup(-0.4, Height, false),
            Err(CdcAbsence::BelowTwentyTwoWeeks)
        );
        assert_eq!(lookup(-0.1, Weight, false), Err(CdcAbsence::BelowTerm));
        assert_eq!(lookup(0.5, Ofc, false), Ok(TableId::CdcInfant));
        assert_eq!(lookup(3.0, Ofc, false), Ok(TableId::CdcInfant));
        assert_eq!(lookup(3.1, Ofc, false), Err(CdcAbsence::OfcAboveThreeYears));
        assert_eq!(
            lookup(1.0, Height, false),
            Ok(TableId::Who2006InfantPre2025)
        );
        assert_eq!(lookup(2.0, Height, true), Ok(TableId::Who2006InfantPre2025));
        assert_eq!(lookup(2.0, Height, false), Ok(TableId::CdcChild));
        assert_eq!(lookup(1.0, Bmi, false), Err(CdcAbsence::BmiBelowTwoYears));
        assert_eq!(
            lookup(20.5, Weight, false),
            Err(CdcAbsence::AboveTwentyYears)
        );
    }
}
