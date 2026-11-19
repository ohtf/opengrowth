// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-FileCopyrightText: Royal College of Paediatrics and Child Health
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The full result for one measurement: ages, comments, SDS, centiles,
//! bands, and errors (the upstream `Measurement` class, `R12`).
//!
//! Upstream records most failures as strings inside the result rather than
//! raising, and later fields still get calculated. That behaviour is part of
//! the API contract and is kept. Two kinds of failure do stop the calculation:
//! missing reference tables (which upstream cannot have), and the failures
//! upstream lets escape as exceptions, which the API reports as a 422.
//!
//! Only UK-WHO is implemented (`R9`).

use jiff::civil::Date;

use crate::advice::{AgeComments, centile_band, prematurity_comments};
use crate::data::{DataError, ReferenceData, TableId};
use crate::dates::{
    self, DateError, Gestation, calendar_age, chronological_decimal_age, corrected_gestational_age,
    estimated_date_delivery, long_date,
};
use crate::pyfmt::python_repr;
use crate::references::{self, Correction};
use crate::round::python_round;
use crate::sds::{CalcError, centile, percentage_median_bmi, sds_for_measurement};
use crate::vocabulary::{GrowthReference, MeasurementMethod, Sex};

/// What to calculate.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MeasurementInput {
    /// Growth reference.
    pub reference: GrowthReference,
    /// Date of birth.
    pub birth_date: Date,
    /// Date of the measurement.
    pub observation_date: Date,
    /// Gestation at birth.
    pub gestation: Gestation,
    /// Sex.
    pub sex: Sex,
    /// What was measured.
    pub method: MeasurementMethod,
    /// The measurement, in cm, kg, or kg/m².
    pub observation_value: f64,
}

/// Birth details, including the estimated date of delivery.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BirthData {
    /// Date of birth.
    pub birth_date: Date,
    /// Gestation at birth.
    pub gestation: Gestation,
    /// Estimated date of delivery.
    pub estimated_date_delivery: Option<Date>,
    /// The EDD written out, for example "Sun 12 April, 2020".
    pub estimated_date_delivery_string: Option<String>,
    /// Sex.
    pub sex: Sex,
}

/// Ages at the measurement, with comments.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementDates {
    /// Date of the measurement.
    pub observation_date: Date,
    /// Chronological decimal age.
    pub chronological_decimal_age: f64,
    /// Corrected decimal age; `None` if the dates are invalid.
    pub corrected_decimal_age: Option<f64>,
    /// Chronological age in words.
    pub chronological_calendar_age: Option<String>,
    /// Corrected age in words; `None` before the EDD.
    pub corrected_calendar_age: Option<String>,
    /// Corrected gestational age in weeks and days, below 42 weeks only.
    pub corrected_gestational_age: Option<(i64, i64)>,
    /// Comments on age correction; `None` if the dates are invalid.
    pub comments: Option<AgeComments>,
    /// Why the corrected age could not be calculated.
    pub corrected_decimal_age_error: Option<String>,
    /// Why the chronological age could not be described.
    pub chronological_decimal_age_error: Option<String>,
}

/// SDS, centiles, bands, and errors for both ages.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CalculatedValues {
    /// SDS at corrected age.
    pub corrected_sds: Option<f64>,
    /// Centile at corrected age, unrounded.
    pub corrected_centile: Option<f64>,
    /// Centile band sentence at corrected age.
    pub corrected_centile_band: Option<String>,
    /// SDS at chronological age.
    pub chronological_sds: Option<f64>,
    /// Centile at chronological age, unrounded.
    pub chronological_centile: Option<f64>,
    /// Centile band sentence at chronological age.
    pub chronological_centile_band: Option<String>,
    /// Why the corrected SDS could not be calculated.
    pub corrected_measurement_error: Option<String>,
    /// Why the chronological SDS could not be calculated.
    pub chronological_measurement_error: Option<String>,
    /// BMI as a percentage of the median at corrected age.
    pub corrected_percentage_median_bmi: Option<f64>,
    /// BMI as a percentage of the median at chronological age.
    pub chronological_percentage_median_bmi: Option<f64>,
}

/// The complete result.
#[derive(Debug, Clone, PartialEq)]
pub struct Measurement {
    /// The reference actually used.
    pub growth_reference: GrowthReference,
    /// Every table read, in order, without duplicates.
    pub tables_read: Vec<TableId>,
    /// Birth details.
    pub birth_data: BirthData,
    /// Ages.
    pub measurement_dates: MeasurementDates,
    /// What was measured.
    pub method: MeasurementMethod,
    /// The measurement.
    pub observation_value: f64,
    /// Why the measurement is implausible, if it is.
    pub observation_value_error: Option<String>,
    /// SDS and centiles.
    pub values: CalculatedValues,
}

/// Why no result object could be produced.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum MeasureError {
    /// A reference table is not loaded. Never substituted.
    #[error(transparent)]
    Data(#[from] DataError),
    /// Upstream would raise here; the API turns it into a 422 with this message.
    #[error("{0}")]
    Raised(String),
}

/// SDS for the measurement validity check, at the upstream limits.
const MAX_HEIGHT_WEIGHT_OFC_SDS: f64 = 8.0;
const MAX_BMI_SDS: f64 = 15.0;

/// Calculates the full result for one measurement.
///
/// # Errors
///
/// See [`MeasureError`].
pub fn measure(
    data: &dyn ReferenceData,
    input: &MeasurementInput,
) -> Result<Measurement, MeasureError> {
    let mut tables_read = Vec::new();
    let measurement_dates = ages(input);
    let birth_data = BirthData {
        birth_date: input.birth_date,
        gestation: input.gestation,
        estimated_date_delivery: estimated_date_delivery(input.birth_date, input.gestation).ok(),
        estimated_date_delivery_string: estimated_date_delivery(input.birth_date, input.gestation)
            .ok()
            .map(long_date),
        sex: input.sex,
    };

    let observation_value_error = match measurement_dates.corrected_decimal_age {
        Some(age) => validate_observation(data, input, age, &mut tables_read)?,
        // Upstream records a Python TypeError text here; not reachable through the API.
        None => None,
    };

    let values = match measurement_dates.corrected_decimal_age {
        None => CalculatedValues {
            corrected_measurement_error: Some("Dates error. Calculations impossible.".to_owned()),
            chronological_measurement_error: Some(
                "Dates error. Calculations impossible.".to_owned(),
            ),
            ..CalculatedValues::default()
        },
        Some(corrected_age) => calculated_values(
            data,
            input,
            corrected_age,
            measurement_dates.chronological_decimal_age,
            &mut tables_read,
        )?,
    };

    tables_read.sort();
    tables_read.dedup();
    Ok(Measurement {
        growth_reference: input.reference,
        tables_read,
        birth_data,
        measurement_dates,
        method: input.method,
        observation_value: input.observation_value,
        observation_value_error,
        values,
    })
}

/// The upstream `__calculate_ages`. Most references correct every
/// gestation; those with the term rule (WHO and CDC) treat 37 to 42 weeks as
/// term and stop correcting at 2 years, by using the chronological age as
/// the corrected age.
fn ages(input: &MeasurementInput) -> MeasurementDates {
    let chronological = chronological_decimal_age(input.birth_date, input.observation_date);
    let weeks = input.gestation.weeks;
    let correction = references::definition(input.reference).correction;
    let corrected =
        dates::corrected_decimal_age(input.birth_date, input.observation_date, input.gestation)
            .map(|age| {
                let term_rules = correction == Correction::TermRule;
                let as_term = (age >= 2.0 && weeks < 37) || (37..=42).contains(&weeks);
                if term_rules && as_term {
                    chronological
                } else {
                    age
                }
            });
    let edd = estimated_date_delivery(input.birth_date, input.gestation).ok();
    let corrected_gestational_age =
        corrected_gestational_age(input.birth_date, input.observation_date, input.gestation);
    let corrected_calendar_age = edd.and_then(|edd| calendar_age(edd, input.observation_date).ok());

    match corrected {
        Ok(corrected_age) => MeasurementDates {
            observation_date: input.observation_date,
            chronological_decimal_age: chronological,
            corrected_decimal_age: Some(corrected_age),
            chronological_calendar_age: calendar_age(input.birth_date, input.observation_date).ok(),
            corrected_calendar_age,
            corrected_gestational_age,
            comments: Some(prematurity_comments(
                chronological,
                corrected_age,
                input.gestation,
                correction,
            )),
            corrected_decimal_age_error: None,
            chronological_decimal_age_error: None,
        },
        Err(error) => MeasurementDates {
            observation_date: input.observation_date,
            chronological_decimal_age: chronological,
            corrected_decimal_age: None,
            chronological_calendar_age: None,
            corrected_calendar_age,
            corrected_gestational_age,
            comments: None,
            corrected_decimal_age_error: Some(date_error_text(&error)),
            chronological_decimal_age_error: None,
        },
    }
}

fn date_error_text(error: &DateError) -> String {
    error.to_string()
}

/// The upstream `__validate_measurement_method`: a message if the
/// measurement is implausible at the corrected age, otherwise `None`.
fn validate_observation(
    data: &dyn ReferenceData,
    input: &MeasurementInput,
    corrected_age: f64,
    tables_read: &mut Vec<TableId>,
) -> Result<Option<String>, MeasureError> {
    let value = python_repr(input.observation_value);
    let age = python_repr(python_round(corrected_age, 1));
    let sds = match sds_for_measurement(
        data,
        input.reference,
        corrected_age,
        input.method,
        input.observation_value,
        input.sex,
    ) {
        Ok(result) => {
            tables_read.push(result.table);
            result.sds
        }
        Err(CalcError::Data(error)) => return Err(error.into()),
        Err(error) => return Ok(Some(error.to_string())),
    };
    let message = match input.method {
        MeasurementMethod::Bmi if sds < -MAX_BMI_SDS => {
            format!(
                "The Body Mass Index measurement of {value} kg/m² is below -15 SD and considered to be an error."
            )
        }
        MeasurementMethod::Bmi if sds > MAX_BMI_SDS => {
            format!(
                "The Body Mass Index measurement of {value} kg/m² is above +15 SD and considered to be an error."
            )
        }
        MeasurementMethod::Height if input.observation_value < 2.0 => {
            "Height/length must be passed in cm, not metres".to_owned()
        }
        MeasurementMethod::Height if sds < -MAX_HEIGHT_WEIGHT_OFC_SDS => format!(
            "The height/length of {value} cm in a child of {age} years is below -8 SD and considered to be an error."
        ),
        MeasurementMethod::Height if sds > MAX_HEIGHT_WEIGHT_OFC_SDS => format!(
            "The height/length of {value} cm in a child of {age} years is above +8 SD and considered to be an error."
        ),
        MeasurementMethod::Weight if sds < -MAX_HEIGHT_WEIGHT_OFC_SDS => format!(
            "The weight of {value} kg is below -8 SD in a child of {age} years and considered to be an error."
        ),
        MeasurementMethod::Weight if sds > MAX_HEIGHT_WEIGHT_OFC_SDS => format!(
            "The weight of {value} kg is above +8 SD in a child of {age} years and considered to be an error. Note that the weight should be supplied in kilograms."
        ),
        MeasurementMethod::Ofc if sds < -MAX_HEIGHT_WEIGHT_OFC_SDS => format!(
            "The head circumference of {value} cm in a child of {age} years is below -8 SD and considered to be an error."
        ),
        MeasurementMethod::Ofc if sds > MAX_HEIGHT_WEIGHT_OFC_SDS => format!(
            "The head circumference of {value} cm in a child of {age} years is above +8 SD and considered to be an error."
        ),
        _ => return Ok(None),
    };
    Ok(Some(message))
}

/// The upstream `sds_and_centile_for_measurement_method`.
fn calculated_values(
    data: &dyn ReferenceData,
    input: &MeasurementInput,
    corrected_age: f64,
    chronological_age: f64,
    tables_read: &mut Vec<TableId>,
) -> Result<CalculatedValues, MeasureError> {
    let definition = references::definition(input.reference);
    let format = (definition.centile_format)(input.method);
    let at = |age: f64| {
        sds_for_measurement(
            data,
            input.reference,
            age,
            input.method,
            input.observation_value,
            input.sex,
        )
    };

    // Upstream catches every exception at corrected age, but only LookupError
    // (no reference data at this age) at chronological age.
    let corrected = if let Some(message) = definition.before_term.filter(|_| corrected_age < 0.0) {
        Err(message.to_owned())
    } else {
        match at(corrected_age) {
            Ok(result) => {
                tables_read.push(result.table);
                Ok(result.sds)
            }
            Err(CalcError::Data(error)) => return Err(error.into()),
            Err(error) => Err(error.to_string()),
        }
    };
    let chronological = match at(chronological_age) {
        Ok(result) => {
            tables_read.push(result.table);
            Ok(result.sds)
        }
        Err(CalcError::Data(error)) => return Err(error.into()),
        Err(error @ CalcError::Absent(_)) => Err(error.to_string()),
        Err(error) => return Err(MeasureError::Raised(error.to_string())),
    };

    let mut values = CalculatedValues::default();
    match corrected {
        Ok(sds) => {
            values.corrected_sds = Some(sds);
            values.corrected_centile = Some(centile(sds));
            values.corrected_centile_band = centile_band(sds, input.method, format);
        }
        Err(message) => values.corrected_measurement_error = Some(message),
    }
    match chronological {
        Ok(sds) => {
            values.chronological_sds = Some(sds);
            values.chronological_centile = Some(centile(sds));
            values.chronological_centile_band = centile_band(sds, input.method, format);
        }
        Err(message) => values.chronological_measurement_error = Some(message),
    }

    if input.method == MeasurementMethod::Bmi {
        let median = |age: f64, tables_read: &mut Vec<TableId>| {
            percentage_median_bmi(
                data,
                input.reference,
                age,
                input.observation_value,
                input.sex,
            )
            .ok()
            .map(|(percent, table)| {
                tables_read.push(table);
                percent
            })
        };
        values.corrected_percentage_median_bmi = median(corrected_age, tables_read);
        values.chronological_percentage_median_bmi = median(chronological_age, tables_read);
    }
    Ok(values)
}
