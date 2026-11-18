// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-FileCopyrightText: Royal College of Paediatrics and Child Health
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Fictional children: a series of measurements that follow an SDS line, for
//! demonstration and testing.
//!
//! Ported from `rcpchgrowth.fictional_child.generate_fictional_child_data`.
//! Every measurement is a full [`Measurement`] of a child born on upstream's
//! fixed date, 11 April 1759, the birthday of François Guéneau de
//! Montbeillard, the subject of the first published growth chart.
//!
//! A series has two parts, kept separate so that a caller can reproduce
//! upstream exactly when an input is missing:
//!
//! - **The [`schedule`]**: ages, SDS, and observation dates. Ages advance by
//!   the interval in decimal years, and dates by the same interval in days
//!   through CPython's `timedelta` ([`python_add_days`]), which drops any
//!   part of a day, so the dates fall behind the ages. A preterm child's
//!   ages are corrected for gestation and the dates are not.
//! - **The measurements**: at each step, the measurement at that SDS and age
//!   ([`raw_measurement`]; none where the reference has no data there),
//!   optionally with random noise, rounded to one decimal place, and then
//!   calculated in full. Rounding follows upstream's float kind: after a
//!   linear table lookup the measurement is a NumPy float ([`PyFloat`]).
//!
//! The engine has no random source of its own: noise is drawn from the
//! `uniform` function the caller supplies, standing in for Python's
//! `random.random()`.

use jiff::civil::{Date, date};

use crate::data::{DataError, ReferenceData};
use crate::dates::{Gestation, TERM_PREGNANCY_DAYS, python_add_days};
use crate::measurement::{MeasureError, Measurement, MeasurementInput, measure};
use crate::pyfmt::python_repr;
use crate::round::{PyFloat, python_round};
use crate::sds::{CalcError, measurement_from_sds};
use crate::vocabulary::{GrowthReference, MeasurementMethod, Sex};

/// Every fictional child's date of birth.
pub const BIRTH_DATE: Date = date(1759, 4, 11);

/// The unit of the interval between measurements.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntervalUnit {
    /// `d`, `day`, or `days`.
    Days,
    /// `w`, `week`, or `weeks`.
    Weeks,
    /// `m`, `month`, or `months`.
    Months,
    /// `y`, `year`, or `years`.
    Years,
}

impl IntervalUnit {
    /// The unit for one of upstream's names.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "d" | "day" | "days" => Some(Self::Days),
            "w" | "week" | "weeks" => Some(Self::Weeks),
            "m" | "month" | "months" => Some(Self::Months),
            "y" | "year" | "years" => Some(Self::Years),
            _ => None,
        }
    }

    /// `value` of this unit in decimal years (upstream `_interval_value_to_years`).
    #[must_use]
    pub fn to_years(self, value: f64) -> f64 {
        match self {
            Self::Days => value / 365.25,
            Self::Weeks => value / 52.0,
            Self::Months => value / 12.0,
            Self::Years => value,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Days => "days",
            Self::Weeks => "weeks",
            Self::Months => "months",
            Self::Years => "years",
        }
    }
}

/// What to generate.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FictionalChildParams {
    /// Growth reference.
    pub reference: GrowthReference,
    /// What is measured.
    pub method: MeasurementMethod,
    /// Sex.
    pub sex: Sex,
    /// Chronological age of the first step, in decimal years.
    pub start_age: f64,
    /// The series stops before this age, in decimal years.
    pub end_age: f64,
    /// Gestation at birth.
    pub gestation: Gestation,
    /// Unit of the interval between steps.
    pub interval_unit: IntervalUnit,
    /// Units between steps; must be positive.
    pub interval_number: i64,
    /// SDS at the first step.
    pub start_sds: f64,
    /// If set, the SDS drifts by this much over the series.
    pub drift_range: Option<f64>,
    /// If set, each measurement gets uniform noise of up to this fraction of
    /// its value, either way.
    pub noise_range: Option<f64>,
}

/// One step of a schedule.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Step {
    /// Age at which the measurement is found, corrected for gestation.
    pub age: f64,
    /// SDS of the measurement.
    pub sds: f64,
    /// Observation date given to the full calculation.
    pub observation_date: Date,
}

/// A series could not be generated.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum FictionalChildError {
    /// Upstream raises here, with this message. The API reports every such
    /// failure as one 422 that does not repeat it.
    #[error("{0}")]
    Invalid(String),
    /// A table is not loaded. Never substituted.
    #[error(transparent)]
    Data(#[from] DataError),
    /// The reference has not been ported yet.
    #[error("the {0} reference is not implemented yet")]
    ReferenceNotImplemented(GrowthReference),
}

impl From<MeasureError> for FictionalChildError {
    fn from(error: MeasureError) -> Self {
        match error {
            MeasureError::Data(error) => Self::Data(error),
            MeasureError::Raised(message) => Self::Invalid(message),
            MeasureError::ReferenceNotImplemented(reference) => {
                Self::ReferenceNotImplemented(reference)
            }
        }
    }
}

/// The steps of a series, in order. Fails, as upstream raises, if the end is
/// not after the start, if the interval is not positive or is longer than
/// the series, or if a date falls outside years 1 to 9999.
///
/// # Errors
///
/// [`FictionalChildError::Invalid`] for the checks made before the first
/// step. The iterator itself yields an error if a later date is out of range.
pub fn schedule(params: &FictionalChildParams) -> Result<Schedule, FictionalChildError> {
    let start = params.start_age;
    if params.end_age <= start {
        return Err(FictionalChildError::Invalid(format!(
            "end_age ({} years) must be greater than start_chronological_age ({} years).",
            python_repr(params.end_age),
            python_repr(start)
        )));
    }
    let date = python_add_days(BIRTH_DATE, start * 365.25).ok_or_else(date_out_of_range)?;
    let correction = if params.gestation.weeks < 40 {
        (params.gestation.total_days() - TERM_PREGNANCY_DAYS) as f64 / 365.25
    } else {
        0.0
    };
    let end = params.end_age + correction;
    let interval = params.interval_unit.to_years(params.interval_number as f64);
    let interval_text = || format!("{} {}", params.interval_number, params.interval_unit.name());
    if params.interval_number <= 0 {
        return Err(FictionalChildError::Invalid(format!(
            "measurement_interval_number must be a positive value; received {}.",
            interval_text()
        )));
    }
    // Upstream's span runs from the uncorrected start to the corrected end.
    let span = end - start;
    if span < interval {
        return Err(FictionalChildError::Invalid(format!(
            "The age range ({} years - {} years) is smaller than the measurement interval ({}); no measurements would be generated.",
            python_repr(params.end_age),
            python_repr(start),
            interval_text()
        )));
    }
    let cycles = (span / interval).floor();
    Ok(Schedule {
        age: start + correction,
        sds: params.start_sds,
        date,
        end,
        interval,
        drift: params.drift_range.map(|range| range / cycles),
        started: false,
        done: false,
    })
}

/// The steps of a series; see [`schedule`].
#[derive(Debug, Clone)]
pub struct Schedule {
    age: f64,
    sds: f64,
    date: Date,
    end: f64,
    interval: f64,
    drift: Option<f64>,
    started: bool,
    done: bool,
}

impl Iterator for Schedule {
    type Item = Result<Step, FictionalChildError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        if self.started {
            // The end of upstream's loop body, which runs after every step,
            // the last included, so a date out of range after it still fails.
            if let Some(drift) = self.drift {
                self.sds = python_round(self.sds + drift, 3);
            }
            self.age += self.interval;
            match python_add_days(self.date, self.interval * 365.25) {
                Some(date) => self.date = date,
                None => {
                    self.done = true;
                    return Some(Err(date_out_of_range()));
                }
            }
        }
        self.started = true;
        if self.age < self.end {
            Some(Ok(Step {
                age: self.age,
                sds: self.sds,
                observation_date: self.date,
            }))
        } else {
            self.done = true;
            None
        }
    }
}

fn date_out_of_range() -> FictionalChildError {
    FictionalChildError::Invalid("date value out of range".to_owned())
}

/// The measurement at a step, before noise and rounding: the upstream
/// `measurement_from_sds`, whose failures upstream skips, so they are `None`.
///
/// # Errors
///
/// Only for a table that is not loaded, or a reference not yet ported.
pub fn raw_measurement(
    data: &dyn ReferenceData,
    params: &FictionalChildParams,
    step: &Step,
) -> Result<Option<PyFloat>, FictionalChildError> {
    let found = measurement_from_sds(
        data,
        params.reference,
        PyFloat::Builtin(step.sds),
        params.method,
        params.sex,
        step.age,
        false,
    );
    match found {
        Ok((value, _)) => Ok(value),
        Err(CalcError::Data(error)) => Err(error.into()),
        Err(CalcError::ReferenceNotImplemented(reference)) => {
            Err(FictionalChildError::ReferenceNotImplemented(reference))
        }
        // Upstream gets an infinite measurement here (CDC BMI at a centile
        // that rounds to 100) and its response then fails with a 500; this
        // fails the series instead (`spec/standard.md` section 6.3).
        Err(CalcError::NotReal) => Err(FictionalChildError::Invalid(
            "the measurement at this SDS is not finite".to_owned(),
        )),
        // Upstream prints the exception and moves on to the next step.
        Err(_) => Ok(None),
    }
}

/// A fictional child's measurements (the upstream `generate_fictional_child_data`).
///
/// `uniform` is called once per noisy measurement, for a value in `[0, 1)`.
///
/// # Errors
///
/// See [`FictionalChildError`]. A full calculation that upstream would raise
/// on fails the whole series, as upstream does.
pub fn fictional_child(
    data: &dyn ReferenceData,
    params: &FictionalChildParams,
    uniform: &mut dyn FnMut() -> f64,
) -> Result<Vec<Measurement>, FictionalChildError> {
    let mut measurements = Vec::new();
    for step in schedule(params)? {
        let step = step?;
        let Some(mut raw) = raw_measurement(data, params, &step)? else {
            continue;
        };
        if let Some(range) = params.noise_range {
            // `raw += random.uniform(-error, error)`, where uniform(a, b) is
            // `a + (b - a) * random()`.
            let error = raw.value() * range;
            let (a, b) = (-error, error);
            let noise = a + (b - a) * uniform();
            raw = raw.map(|value| value + noise);
        }
        let input = MeasurementInput {
            reference: params.reference,
            birth_date: BIRTH_DATE,
            observation_date: step.observation_date,
            gestation: params.gestation,
            sex: params.sex,
            method: params.method,
            observation_value: raw.round(1).value(),
        };
        measurements.push(measure(data, &input)?);
    }
    Ok(measurements)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn params() -> FictionalChildParams {
        FictionalChildParams {
            reference: GrowthReference::UkWho,
            method: MeasurementMethod::Height,
            sex: Sex::Male,
            start_age: 0.0,
            end_age: 20.0,
            gestation: Gestation::TERM,
            interval_unit: IntervalUnit::Months,
            interval_number: 20,
            start_sds: 0.0,
            drift_range: None,
            noise_range: None,
        }
    }

    #[test]
    fn default_schedule_has_twelve_steps_whose_dates_fall_behind_the_ages() {
        let steps: Vec<Step> = schedule(&params()).unwrap().map(Result::unwrap).collect();
        assert_eq!(steps.len(), 12);
        // 20 months is 608.75 days; each date step drops the 0.75.
        assert_eq!(steps[1].observation_date, date(1760, 12, 9));
        assert_eq!(
            steps[11].observation_date,
            BIRTH_DATE + jiff::Span::new().days(11 * 608)
        );
    }

    #[test]
    fn a_series_that_cannot_be_made_is_refused() {
        let bad = [
            FictionalChildParams {
                end_age: 0.0,
                ..params()
            },
            FictionalChildParams {
                interval_number: 0,
                ..params()
            },
            FictionalChildParams {
                interval_unit: IntervalUnit::Years,
                interval_number: 21,
                ..params()
            },
        ];
        for params in bad {
            assert!(matches!(
                schedule(&params),
                Err(FictionalChildError::Invalid(_))
            ));
        }
    }

    #[test]
    fn a_date_beyond_9999_fails_even_after_the_last_step() {
        let params = FictionalChildParams {
            start_age: 19.0,
            end_age: 9000.0,
            interval_unit: IntervalUnit::Years,
            interval_number: 1000,
            ..params()
        };
        let last = schedule(&params).unwrap().last().unwrap();
        assert!(matches!(last, Err(FictionalChildError::Invalid(_))));
    }
}
