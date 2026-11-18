// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-FileCopyrightText: Royal College of Paediatrics and Child Health
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The WHO reference: which table serves an age, and where there is none.
//!
//! Ported from `rcpchgrowth.who`. WHO joins three tables: the WHO 2006
//! growth standard from birth to 2 years and from 2 years to 1856 days
//! (5.08 years), both daily, then the WHO 2007 growth reference, monthly,
//! to 19 years. Unlike UK-WHO there is no preterm table: WHO has no data
//! below term. Absence is checked before the table is chosen, as upstream.

use crate::data::TableId;
use crate::vocabulary::MeasurementMethod;

/// The upper end of the WHO 2006 child table: 1856 days.
pub const WHO_2006_UPPER_THRESHOLD: f64 = 1856.0 / 365.25;
/// The WHO 2006 infant table ends, and the child table starts, at 2 years.
pub const WHO_CHILD_LOWER_THRESHOLD: f64 = 2.0;
/// The WHO 2007 reference ends at 19 years.
pub const WHO_2007_UPPER_THRESHOLD: f64 = 19.0;
/// WHO weight data stop at 10 years.
pub const WEIGHT_UPPER: f64 = 10.0;

/// No WHO data exists for this request. Each message is upstream's
/// `LookupError` text, which reaches API responses unchanged.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum WhoAbsence {
    /// Below term (corrected age 0).
    #[error("WHO data does not exist below term.")]
    BelowTerm,
    /// Above 19 years.
    #[error("WHO data does not exist above 19 years.")]
    AboveNineteenYears,
    /// Weight above 10 years.
    #[error("WHO weight data does not exist in children over 10 y of age.")]
    WeightAboveTenYears,
    /// Head circumference above the WHO 2006 standard.
    #[error("WHO head circumference data does not exist in children over 5 y of age.")]
    OfcAboveFiveYears,
    /// No table above 19 years (upstream `who_reference`, which the checks
    /// above make unreachable in sequence).
    #[error("There are no WHO reference data above the age of 19 years.")]
    NoTableAboveNineteenYears,
}

/// Whether WHO lacks data for this measurement at this age (the upstream
/// `reference_data_absent`). Checked before [`table_for_age`], as upstream.
#[must_use]
pub fn data_absent(age: f64, method: MeasurementMethod) -> Option<WhoAbsence> {
    if age < 0.0 {
        Some(WhoAbsence::BelowTerm)
    } else if age > 19.0 {
        Some(WhoAbsence::AboveNineteenYears)
    } else if method == MeasurementMethod::Weight && age > WEIGHT_UPPER {
        Some(WhoAbsence::WeightAboveTenYears)
    } else if method == MeasurementMethod::Ofc && age > WHO_2006_UPPER_THRESHOLD {
        Some(WhoAbsence::OfcAboveFiveYears)
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
pub fn table_for_age(age: f64, youngest_at_boundary: bool) -> Result<TableId, WhoAbsence> {
    if age <= WHO_2006_UPPER_THRESHOLD {
        if (age == 2.0 && youngest_at_boundary) || age < WHO_CHILD_LOWER_THRESHOLD {
            Ok(TableId::Who2006Infant)
        } else {
            Ok(TableId::Who2006Child)
        }
    } else if age <= WHO_2007_UPPER_THRESHOLD {
        Ok(TableId::Who2007Child)
    } else {
        Err(WhoAbsence::NoTableAboveNineteenYears)
    }
}

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
            Some(WhoAbsence::BelowTerm)
        );
        assert_eq!(
            data_absent(19.5, MeasurementMethod::Weight),
            Some(WhoAbsence::AboveNineteenYears)
        );
        assert_eq!(
            data_absent(10.5, MeasurementMethod::Weight),
            Some(WhoAbsence::WeightAboveTenYears)
        );
        assert_eq!(
            data_absent(5.1, MeasurementMethod::Ofc),
            Some(WhoAbsence::OfcAboveFiveYears)
        );
        assert_eq!(data_absent(5.1, MeasurementMethod::Bmi), None);
    }
}
