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

/// `date + timedelta(days=days)` as CPython computes it, or `None` where
/// CPython raises `OverflowError` or `ValueError`.
///
/// CPython converts the float to whole microseconds, rounding half to even
/// (`accum` and `delta_new` in `_datetimemodule.c`), and adding a timedelta
/// to a date then uses only its whole days. So a step of
/// `7 / 365.25 * 365.25` days, which is `6.999999999999999`, still moves the
/// date 7 days. Fails if `days` is not finite, if the timedelta would exceed
/// 999,999,999 days, or if the result is outside years 1 to 9999.
#[must_use]
pub fn python_add_days(date: Date, days: f64) -> Option<Date> {
    let whole_days = python_timedelta_days(days)?;
    let span = jiff::Span::new().try_days(whole_days).ok()?;
    let result = date.checked_add(span).ok()?;
    (1..=9999).contains(&result.year()).then_some(result)
}

/// `timedelta(days=days).days`, or `None` where CPython raises.
fn python_timedelta_days(days: f64) -> Option<i64> {
    const US_PER_DAY: i128 = 86_400_000_000;
    const MAX_DELTA_DAYS: i128 = 999_999_999;
    // `PyLong_FromDouble` raises for infinity and NaN, and anything this large
    // is far beyond the timedelta limit.
    if !days.is_finite() || days.abs() >= 1e12 {
        return None;
    }
    // `accum`: the integer part exactly, the fraction in floating point.
    let int_part = days.trunc();
    let fraction = days - int_part;
    let mut microseconds = int_part as i128 * US_PER_DAY;
    let mut leftover = 0.0;
    if fraction != 0.0 {
        let scaled = 86_400_000_000.0 * fraction;
        let whole = scaled.trunc();
        microseconds += whole as i128;
        leftover += scaled - whole;
    }
    // `delta_new`: round the leftover microseconds, half to even.
    if leftover != 0.0 {
        let mut whole_us = leftover.round();
        if (whole_us - leftover).abs() == 0.5 {
            let odd = (microseconds & 1) as f64;
            whole_us = 2.0 * ((leftover + odd) * 0.5).round() - odd;
        }
        microseconds += whole_us as i128;
    }
    let whole_days = microseconds.div_euclid(US_PER_DAY);
    if whole_days.abs() > MAX_DELTA_DAYS {
        return None;
    }
    i64::try_from(whole_days).ok()
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

/// Calendar difference in whole years, months, and days, as
/// `dateutil.relativedelta(later, earlier)` computes it for dates.
///
/// Months are counted first, clamping the day to the end of shorter
/// months (31 January plus one month is 28 or 29 February), then the
/// remaining days.
fn relative_delta(earlier: Date, later: Date) -> (i64, i64, i64) {
    let add_months = |months: i64| -> Date {
        let total = i64::from(earlier.year()) * 12 + i64::from(earlier.month()) - 1 + months;
        let year = i16::try_from(total.div_euclid(12)).expect("year in range");
        let month = i8::try_from(total.rem_euclid(12) + 1).expect("month in range");
        let first = Date::new(year, month, 1).expect("valid first of month");
        let day = earlier.day().min(first.days_in_month());
        Date::new(year, month, day).expect("clamped day is valid")
    };
    let mut months = (i64::from(later.year()) - i64::from(earlier.year())) * 12
        + (i64::from(later.month()) - i64::from(earlier.month()));
    let mut anchor = add_months(months);
    while later < anchor {
        months -= 1;
        anchor = add_months(months);
    }
    let days = days_between(anchor, later);
    (months / 12, months % 12, days)
}

/// Calendar age as words, for example "8 years and 2 months" or
/// "1 month, 1 week and 4 days" (the upstream `chronological_calendar_age`).
///
/// Pass the EDD as `birth_date` for a corrected calendar age.
///
/// # Errors
///
/// [`DateError::BirthAfterObservation`] if `birth_date` is after `observation_date`.
pub fn calendar_age(birth_date: Date, observation_date: Date) -> Result<String, DateError> {
    if birth_date > observation_date {
        return Err(DateError::BirthAfterObservation);
    }
    let (years, months, days) = relative_delta(birth_date, observation_date);
    let weeks = days / 7;
    let plural = |n: i64, unit: &str| {
        if n == 1 {
            format!("{n} {unit}")
        } else {
            format!("{n} {unit}s")
        }
    };

    // The branch structure follows upstream exactly, including that a lone
    // day is shown as "1 day" but 2 to 6 days without a whole week are shown
    // as "N days", and that weeks are only mentioned when days exceed 1.
    let mut parts = Vec::new();
    if years >= 1 {
        parts.push(plural(years, "year"));
    }
    if months >= 1 {
        parts.push(plural(months, "month"));
    }
    if days == 1 {
        parts.push(plural(days, "day"));
    }
    if days > 1 {
        if weeks > 0 {
            let remaining_days = days - weeks * 7;
            parts.push(plural(weeks, "week"));
            if remaining_days >= 1 {
                parts.push(plural(remaining_days, "day"));
            }
        } else {
            parts.push(plural(days, "day"));
        }
    }
    Ok(match parts.as_slice() {
        [] if birth_date == observation_date => "Birth date".to_owned(),
        [] => String::new(),
        [only] => only.clone(),
        [init @ .., last] => format!("{} and {last}", init.join(", ")),
    })
}

/// Corrected gestational age in completed weeks and days, while the baby is
/// younger than 42 weeks corrected (the upstream `corrected_gestational_age`).
/// `None` from two weeks after the EDD onwards.
#[must_use]
pub fn corrected_gestational_age(
    birth_date: Date,
    observation_date: Date,
    gestation: Gestation,
) -> Option<(i64, i64)> {
    let edd = estimated_date_delivery(birth_date, gestation).ok()?;
    let forty_two_weeks = edd.checked_add(14.days()).ok()?;
    if observation_date >= forty_two_weeks {
        return None;
    }
    let days_since_conception = days_between(birth_date, observation_date) + gestation.total_days();
    let weeks = days_since_conception.div_euclid(7);
    let days = days_since_conception - weeks * 7;
    if (weeks == 42 && days > 0) || weeks > 42 {
        return None;
    }
    Some((weeks, days))
}

/// A date as Python's `strftime('%a %d %B, %Y')` writes it in the C locale,
/// for example "Sun 12 April, 2020".
#[must_use]
pub fn long_date(date: Date) -> String {
    use jiff::civil::Weekday;
    const MONTHS: [&str; 12] = [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];
    let weekday = match date.weekday() {
        Weekday::Monday => "Mon",
        Weekday::Tuesday => "Tue",
        Weekday::Wednesday => "Wed",
        Weekday::Thursday => "Thu",
        Weekday::Friday => "Fri",
        Weekday::Saturday => "Sat",
        Weekday::Sunday => "Sun",
    };
    let month = MONTHS[usize::try_from(date.month() - 1).expect("month 1 to 12")];
    format!("{weekday} {:02} {month}, {}", date.day(), date.year())
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
    fn calendar_age_matches_upstream_phrasing() {
        // Expected strings are what rcpchgrowth 4.6.5 returns for the same dates.
        let cases = [
            ((2020, 4, 12), (2028, 6, 12), "8 years and 2 months"),
            ((2026, 10, 6), (2026, 11, 17), "1 month, 1 week and 4 days"),
            ((2020, 1, 1), (2020, 1, 1), "Birth date"),
            ((2020, 5, 15), (2020, 5, 16), "1 day"),
            ((2020, 5, 15), (2020, 5, 22), "1 week"),
            ((2020, 5, 15), (2020, 5, 23), "1 week and 1 day"),
            ((2020, 5, 15), (2020, 5, 19), "4 days"),
            ((2020, 1, 31), (2020, 2, 29), "1 month"),
            ((2020, 2, 29), (2021, 2, 28), "1 year"),
        ];
        for ((by, bm, bd), (oy, om, od), expected) in cases {
            assert_eq!(
                calendar_age(date(by, bm, bd), date(oy, om, od)).unwrap(),
                expected
            );
        }
        assert!(calendar_age(date(2020, 1, 2), date(2020, 1, 1)).is_err());
    }

    #[test]
    fn corrected_gestational_age_stops_after_42_weeks() {
        let preterm = Gestation::new(27, 3).unwrap();
        assert_eq!(
            corrected_gestational_age(date(2026, 10, 6), date(2026, 11, 17), preterm),
            Some((33, 3))
        );
        assert_eq!(
            corrected_gestational_age(date(2020, 1, 1), date(2020, 3, 1), Gestation::TERM),
            None
        );
        assert_eq!(long_date(date(2020, 4, 12)), "Sun 12 April, 2020");
        assert_eq!(long_date(date(2027, 1, 2)), "Sat 02 January, 2027");
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
