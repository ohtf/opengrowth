// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-FileCopyrightText: Royal College of Paediatrics and Child Health
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Decimal ages, gestation, and the estimated date of delivery.
//!
//! Ported from `rcpchgrowth.date_calculations`. A decimal age is whole days
//! divided by 365.25. A corrected age is measured from the estimated date of
//! delivery (EDD) rather than the birth date, so it is negative before term
//! and larger than the chronological age for babies born after 40 weeks.
//!
//! This module applies no reference-specific rules. The CDC and WHO rule
//! that stops correcting at term and at two years belongs to those
//! references and is applied by the caller.

use jiff::ToSpan;
use jiff::civil::Date;
use serde::{Deserialize, Serialize};

/// Days in a decimal year.
pub const DAYS_PER_YEAR: f64 = 365.25;

/// Length of a term pregnancy in days (40 weeks).
pub const TERM_PREGNANCY_DAYS: i64 = 40 * 7;

/// Gestational age at birth, in completed weeks plus extra days.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Gestation {
    /// Completed weeks.
    pub weeks: u8,
    /// Days beyond the completed weeks, 0 to 6.
    pub days: u8,
}

/// A gestation could not be parsed or is outside the plausible range.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum GestationError {
    /// Not in the form `W+D`, such as `32+4`.
    #[error("gestation `{0}` is not in the form WEEKS+DAYS, for example 32+4")]
    Format(String),
    /// Supplementary days must be 0 to 6.
    #[error("gestation days must be 0 to 6, got {0}")]
    Days(u8),
    /// Outside the 22 to 44 weeks the upstream API accepts.
    #[error("gestation weeks must be 22 to 44, got {0}")]
    Weeks(u8),
}

impl Gestation {
    /// Earliest gestation the upstream API accepts, in weeks.
    pub const MIN_WEEKS: u8 = 22;
    /// Latest gestation the upstream API accepts, in weeks.
    pub const MAX_WEEKS: u8 = 44;
    /// 40+0 weeks.
    pub const TERM: Self = Self { weeks: 40, days: 0 };

    /// A gestation within 22+0 to 44+6 weeks.
    ///
    /// # Errors
    ///
    /// If weeks or days are out of range.
    pub fn new(weeks: u8, days: u8) -> Result<Self, GestationError> {
        if days > 6 {
            return Err(GestationError::Days(days));
        }
        if !(Self::MIN_WEEKS..=Self::MAX_WEEKS).contains(&weeks) {
            return Err(GestationError::Weeks(weeks));
        }
        Ok(Self { weeks, days })
    }

    /// Total length of the pregnancy in days.
    #[must_use]
    pub fn total_days(self) -> i64 {
        i64::from(self.weeks) * 7 + i64::from(self.days)
    }
}

impl std::str::FromStr for Gestation {
    type Err = GestationError;

    /// Parses `W+D` (such as `32+4`) or plain `W`.
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let format_error = || GestationError::Format(text.to_owned());
        let (weeks, days) = text.trim().split_once('+').unwrap_or((text.trim(), "0"));
        let weeks = weeks.trim().parse().map_err(|_| format_error())?;
        let days = days.trim().parse().map_err(|_| format_error())?;
        Self::new(weeks, days)
    }
}

impl std::fmt::Display for Gestation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}+{}", self.weeks, self.days)
    }
}

/// A date calculation could not be performed.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DateError {
    /// The upstream message, which reaches API responses unchanged.
    #[error("Birth date cannot be after the date of observation.")]
    BirthAfterObservation,
    /// The EDD falls outside the representable calendar.
    #[error("estimated date of delivery is out of range")]
    OutOfRange,
}

/// Whole days from `from` to `to`; negative if `to` is earlier.
#[must_use]
pub fn days_between(from: Date, to: Date) -> i64 {
    from.duration_until(to).as_hours() / 24
}

/// Decimal age from two dates. Negative if the observation precedes birth,
/// as upstream; callers that need a refusal check [`corrected_decimal_age`].
#[must_use]
pub fn chronological_decimal_age(birth_date: Date, observation_date: Date) -> f64 {
    days_between(birth_date, observation_date) as f64 / DAYS_PER_YEAR
}

/// Estimated date of delivery: the birth date moved by the days the baby was
/// born before (or after) 40 weeks.
///
/// # Errors
///
/// [`DateError::OutOfRange`] if the result is not a representable date.
pub fn estimated_date_delivery(birth_date: Date, gestation: Gestation) -> Result<Date, DateError> {
    let correction_days = TERM_PREGNANCY_DAYS - gestation.total_days();
    birth_date
        .checked_add(correction_days.days())
        .map_err(|_| DateError::OutOfRange)
}

/// Decimal age measured from the estimated date of delivery.
///
/// # Errors
///
/// [`DateError::BirthAfterObservation`] if the birth date is after the
/// observation date.
pub fn corrected_decimal_age(
    birth_date: Date,
    observation_date: Date,
    gestation: Gestation,
) -> Result<f64, DateError> {
    if birth_date > observation_date {
        return Err(DateError::BirthAfterObservation);
    }
    let edd = estimated_date_delivery(birth_date, gestation)?;
    Ok(chronological_decimal_age(edd, observation_date))
}

#[cfg(test)]
mod tests {
    use super::*;
    use jiff::civil::date;

    #[test]
    fn decimal_ages_use_365_25_day_years() {
        assert_eq!(
            chronological_decimal_age(date(2020, 1, 1), date(2020, 1, 1)),
            0.0
        );
        assert_eq!(
            chronological_decimal_age(date(2020, 1, 1), date(2021, 1, 1)),
            366.0 / 365.25
        );
        assert!(chronological_decimal_age(date(2021, 1, 1), date(2020, 1, 1)) < 0.0);
    }

    #[test]
    fn corrected_age_follows_the_edd() {
        let birth = date(2012, 1, 28);
        let observed = date(2013, 4, 26);
        // From the R vectors: 41+2 weeks, chronological 1.242984257, corrected 1.267624914.
        let gestation = Gestation::new(41, 2).unwrap();
        let chronological = chronological_decimal_age(birth, observed);
        let corrected = corrected_decimal_age(birth, observed, gestation).unwrap();
        assert!((chronological - 1.242_984_257).abs() < 1e-9);
        assert!((corrected - 1.267_624_914).abs() < 1e-9);

        let preterm = Gestation::new(28, 0).unwrap();
        assert_eq!(
            estimated_date_delivery(birth, preterm).unwrap(),
            date(2012, 4, 21)
        );
        assert_eq!(
            corrected_decimal_age(birth, date(2012, 1, 29), preterm).unwrap(),
            (1.0 - 84.0) / 365.25
        );
    }

    #[test]
    fn birth_after_observation_is_refused_with_the_upstream_message() {
        let err =
            corrected_decimal_age(date(2020, 1, 2), date(2020, 1, 1), Gestation::TERM).unwrap_err();
        assert_eq!(
            err.to_string(),
            "Birth date cannot be after the date of observation."
        );
    }

    #[test]
    fn gestation_parsing_and_range() {
        assert_eq!(
            "32+4".parse::<Gestation>().unwrap(),
            Gestation { weeks: 32, days: 4 }
        );
        assert_eq!("40".parse::<Gestation>().unwrap(), Gestation::TERM);
        assert!("21+6".parse::<Gestation>().is_err());
        assert!("45+0".parse::<Gestation>().is_err());
        assert!("40+7".parse::<Gestation>().is_err());
        assert!("forty".parse::<Gestation>().is_err());
    }
}
