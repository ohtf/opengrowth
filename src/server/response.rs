// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Response bodies in the upstream server's shape.
//!
//! Field order is the order of the upstream Pydantic response models
//! (`schemas/response_schema_classes.py`), which is what FastAPI writes,
//! not the order of the engine's own dictionaries. Every optional field is
//! written, as `null` when empty. The four centile fields with upstream's
//! `rounded_centile` validator, and the plottable centiles, are rounded to
//! four decimal places here and nowhere else.

use crate::advice::AgeComments;
use crate::measurement::Measurement;
use crate::round::python_round;
use crate::{ENGINE_NAME, ENGINE_VERSION, MeasurementMethod, Sex};
use serde::Serialize;
use serde_json::Value;

use crate::server::validate::{FieldError, LocItem, ObservationFields};

/// The commit stamped into release builds; `"unknown"` otherwise (`spec/provenance.md` section 4).
pub const COMMIT: &str = match option_env!("OPENGROWTH_COMMIT") {
    Some(commit) => commit,
    None => "unknown",
};

/// The server's own identity in provenance.
pub const API_SERVER_NAME: &str = "opengrowth-server";

/// One table read for a calculation.
#[derive(Debug, Clone, Serialize)]
pub struct TableUsed {
    /// Table name.
    pub table: &'static str,
    /// File it was loaded from, relative to the data directory.
    pub file: &'static str,
    /// SHA-256 of the verified file.
    pub sha256: &'static str,
    /// `embedded` or `imported`.
    pub source: &'static str,
}

#[derive(Debug, Serialize)]
struct Identity {
    name: &'static str,
    version: &'static str,
    commit: &'static str,
}

/// `Provenance`, plus the additive `reference_data` list.
#[derive(Debug, Serialize)]
pub struct Provenance {
    growth_reference: &'static str,
    calculation_engine: Identity,
    api_server: Identity,
    reference_data: Vec<TableUsed>,
}

impl Provenance {
    /// Provenance for a calculation against `growth_reference` that read `tables`.
    #[must_use]
    pub fn new(growth_reference: &'static str, tables: Vec<TableUsed>) -> Self {
        let identity = |name| Identity {
            name,
            version: ENGINE_VERSION,
            commit: COMMIT,
        };
        Self {
            growth_reference,
            calculation_engine: identity(ENGINE_NAME),
            api_server: identity(API_SERVER_NAME),
            reference_data: tables,
        }
    }
}

#[derive(Debug, Serialize)]
struct BirthData {
    birth_date: String,
    gestation_weeks: Option<i64>,
    gestation_days: Option<i64>,
    estimated_date_delivery: Option<String>,
    estimated_date_delivery_string: Option<String>,
    sex: &'static str,
}

#[derive(Debug, Serialize)]
struct CorrectedGestationalAge {
    corrected_gestation_weeks: Option<i64>,
    corrected_gestation_days: Option<i64>,
}

#[derive(Debug, Serialize)]
struct Comments<'a> {
    clinician_corrected_decimal_age_comment: Option<&'a str>,
    lay_corrected_decimal_age_comment: Option<&'a str>,
    clinician_chronological_decimal_age_comment: Option<&'a str>,
    lay_chronological_decimal_age_comment: Option<&'a str>,
}

#[derive(Debug, Serialize)]
struct MeasurementDates<'a> {
    observation_date: String,
    chronological_decimal_age: f64,
    corrected_decimal_age: Option<f64>,
    chronological_calendar_age: Option<&'a str>,
    corrected_calendar_age: Option<&'a str>,
    corrected_gestational_age: CorrectedGestationalAge,
    comments: Comments<'a>,
    corrected_decimal_age_error: Option<&'a str>,
    chronological_decimal_age_error: Option<&'a str>,
}

#[derive(Debug, Serialize)]
struct ChildObservationValue<'a> {
    measurement_method: &'static str,
    observation_value: f64,
    observation_value_error: Option<&'a str>,
}

#[derive(Debug, Serialize)]
struct MeasurementCalculatedValues<'a> {
    corrected_sds: Option<f64>,
    corrected_centile: Option<f64>,
    corrected_centile_band: Option<&'a str>,
    chronological_sds: Option<f64>,
    chronological_centile: Option<f64>,
    chronological_centile_band: Option<&'a str>,
    corrected_measurement_error: Option<&'a str>,
    chronological_measurement_error: Option<&'a str>,
    corrected_percentage_median_bmi: Option<f64>,
    chronological_percentage_median_bmi: Option<f64>,
}

/// `ChronologicalDecimalAgeData` and `CorrectedDecimalAgeData`, which differ
/// only by `corrected_gestational_age`.
#[derive(Debug, Serialize)]
struct PlotPoint<'a> {
    x: Option<f64>,
    y: Option<f64>,
    b: Option<f64>,
    centile: Option<f64>,
    sds: Option<f64>,
    bone_age_label: Option<&'a str>,
    events_text: Option<&'a [Value]>,
    bone_age_type: Option<&'a str>,
    bone_age_sds: Option<f64>,
    bone_age_centile: Option<f64>,
    observation_error: Option<&'a str>,
    age_type: &'static str,
    calendar_age: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    corrected_gestational_age: Option<&'a str>,
    lay_comment: Option<&'a str>,
    clinician_comment: Option<&'a str>,
    age_error: Option<&'a str>,
    centile_band: Option<&'a str>,
    observation_value_error: Option<&'a str>,
}

#[derive(Debug, Serialize)]
struct PlotPair<'a> {
    chronological_decimal_age_data: PlotPoint<'a>,
    corrected_decimal_age_data: PlotPoint<'a>,
}

#[derive(Debug, Serialize)]
struct PlottableData<'a> {
    centile_data: PlotPair<'a>,
    sds_data: PlotPair<'a>,
}

#[derive(Debug, Serialize)]
struct BoneAge<'a> {
    bone_age: Option<f64>,
    bone_age_type: Option<&'a str>,
    bone_age_sds: Option<f64>,
    bone_age_centile: Option<f64>,
    bone_age_text: Option<&'a str>,
}

#[derive(Debug, Serialize)]
struct EventsData<'a> {
    events_text: Option<&'a [Value]>,
}

/// `MeasurementObject`.
#[derive(Debug, Serialize)]
pub struct MeasurementObject<'a> {
    provenance: Provenance,
    birth_data: BirthData,
    measurement_dates: MeasurementDates<'a>,
    child_observation_value: ChildObservationValue<'a>,
    measurement_calculated_values: MeasurementCalculatedValues<'a>,
    plottable_data: PlottableData<'a>,
    bone_age: BoneAge<'a>,
    events_data: EventsData<'a>,
}

/// `rounded_centile`: four decimal places, ties to even, as Python `round`.
fn rounded(centile: Option<f64>) -> Option<f64> {
    centile.map(|c| python_round(c, 4))
}

fn sex_text(sex: Sex) -> &'static str {
    sex.as_str()
}

fn method_text(method: MeasurementMethod) -> &'static str {
    method.as_str()
}

/// What the request sent for gestation, echoed in `birth_data`.
#[derive(Debug, Clone, Copy)]
pub struct EchoedGestation {
    /// Weeks as sent, after defaults.
    pub weeks: Option<i64>,
    /// Days as sent, after defaults.
    pub days: Option<i64>,
}

impl<'a> MeasurementObject<'a> {
    /// Builds the response for a calculated measurement and the request's
    /// pass-through fields.
    #[must_use]
    pub fn new(
        measurement: &'a Measurement,
        observation: &'a ObservationFields,
        gestation: EchoedGestation,
        provenance: Provenance,
        corrected_gestational_age_text: &'a str,
    ) -> Self {
        let dates = &measurement.measurement_dates;
        let values = &measurement.values;
        let comments: Option<&AgeComments> = dates.comments.as_ref();
        let comment = |pick: fn(&AgeComments) -> &str| comments.map(pick);
        let events = observation.events_text.as_deref();
        let bone_age_type = observation.bone_age_type.as_deref();
        let bone_age_text = observation.bone_age_text.as_deref();

        let point =
            |corrected: bool, y: Option<f64>, sds: Option<f64>, observation_error| PlotPoint {
                x: Some(if corrected {
                    dates.corrected_decimal_age.unwrap_or(f64::NAN)
                } else {
                    dates.chronological_decimal_age
                }),
                y,
                b: observation.bone_age,
                centile: rounded(if corrected {
                    values.corrected_centile
                } else {
                    values.chronological_centile
                }),
                sds,
                bone_age_label: bone_age_text,
                events_text: events,
                bone_age_type,
                bone_age_sds: observation.bone_age_sds,
                bone_age_centile: observation.bone_age_centile,
                observation_error,
                age_type: if corrected {
                    "corrected_age"
                } else {
                    "chronological_age"
                },
                calendar_age: if corrected {
                    dates.corrected_calendar_age.as_deref()
                } else {
                    dates.chronological_calendar_age.as_deref()
                },
                corrected_gestational_age: corrected.then_some(corrected_gestational_age_text),
                lay_comment: if corrected {
                    comment(|c| &c.lay_corrected)
                } else {
                    comment(|c| &c.lay_chronological)
                },
                clinician_comment: if corrected {
                    comment(|c| &c.clinician_corrected)
                } else {
                    comment(|c| &c.clinician_chronological)
                },
                // Upstream reports the corrected-age error on both branches.
                age_error: dates.corrected_decimal_age_error.as_deref(),
                centile_band: if corrected {
                    values.corrected_centile_band.as_deref()
                } else {
                    values.chronological_centile_band.as_deref()
                },
                observation_value_error: if corrected {
                    values.corrected_measurement_error.as_deref()
                } else {
                    values.chronological_measurement_error.as_deref()
                },
            };
        let observed = Some(measurement.observation_value);
        let observation_error = measurement.observation_value_error.as_deref();

        let (corrected_gestation_weeks, corrected_gestation_days) = dates
            .corrected_gestational_age
            .map_or((None, None), |(w, d)| (Some(w), Some(d)));
        let birth = &measurement.birth_data;
        Self {
            provenance,
            birth_data: BirthData {
                birth_date: birth.birth_date.to_string(),
                gestation_weeks: gestation.weeks,
                gestation_days: gestation.days,
                estimated_date_delivery: birth.estimated_date_delivery.map(|d| d.to_string()),
                estimated_date_delivery_string: birth.estimated_date_delivery_string.clone(),
                sex: sex_text(birth.sex),
            },
            measurement_dates: MeasurementDates {
                observation_date: dates.observation_date.to_string(),
                chronological_decimal_age: dates.chronological_decimal_age,
                corrected_decimal_age: dates.corrected_decimal_age,
                chronological_calendar_age: dates.chronological_calendar_age.as_deref(),
                corrected_calendar_age: dates.corrected_calendar_age.as_deref(),
                corrected_gestational_age: CorrectedGestationalAge {
                    corrected_gestation_weeks,
                    corrected_gestation_days,
                },
                comments: Comments {
                    clinician_corrected_decimal_age_comment: comment(|c| &c.clinician_corrected),
                    lay_corrected_decimal_age_comment: comment(|c| &c.lay_corrected),
                    clinician_chronological_decimal_age_comment: comment(|c| {
                        &c.clinician_chronological
                    }),
                    lay_chronological_decimal_age_comment: comment(|c| &c.lay_chronological),
                },
                corrected_decimal_age_error: dates.corrected_decimal_age_error.as_deref(),
                chronological_decimal_age_error: dates.chronological_decimal_age_error.as_deref(),
            },
            child_observation_value: ChildObservationValue {
                measurement_method: method_text(measurement.method),
                observation_value: measurement.observation_value,
                observation_value_error: observation_error,
            },
            measurement_calculated_values: MeasurementCalculatedValues {
                corrected_sds: values.corrected_sds,
                corrected_centile: rounded(values.corrected_centile),
                corrected_centile_band: values.corrected_centile_band.as_deref(),
                chronological_sds: values.chronological_sds,
                chronological_centile: rounded(values.chronological_centile),
                chronological_centile_band: values.chronological_centile_band.as_deref(),
                corrected_measurement_error: values.corrected_measurement_error.as_deref(),
                chronological_measurement_error: values.chronological_measurement_error.as_deref(),
                corrected_percentage_median_bmi: values.corrected_percentage_median_bmi,
                chronological_percentage_median_bmi: values.chronological_percentage_median_bmi,
            },
            plottable_data: PlottableData {
                centile_data: PlotPair {
                    chronological_decimal_age_data: point(
                        false,
                        observed,
                        values.chronological_sds,
                        observation_error,
                    ),
                    corrected_decimal_age_data: point(
                        true,
                        observed,
                        values.corrected_sds,
                        observation_error,
                    ),
                },
                // Upstream's SDS plot dictionaries have no `sds` or
                // `observation_error` keys, so the model writes null.
                sds_data: PlotPair {
                    chronological_decimal_age_data: point(
                        false,
                        values.chronological_sds,
                        None,
                        None,
                    ),
                    corrected_decimal_age_data: point(true, values.corrected_sds, None, None),
                },
            },
            bone_age: BoneAge {
                bone_age: observation.bone_age,
                bone_age_type,
                bone_age_sds: observation.bone_age_sds,
                bone_age_centile: observation.bone_age_centile,
                bone_age_text,
            },
            events_data: EventsData {
                events_text: events,
            },
        }
    }
}

/// An error raised by a route handler: `format_error` in upstream `routers/utils.py`.
/// Key order is `loc`, `msg`, `type`, `input`.
#[derive(Debug, Clone, Serialize)]
pub struct RouteError {
    /// Location; always `["body"]` upstream.
    pub loc: Vec<LocItem>,
    /// Message.
    pub msg: String,
    /// `value_error` or `lookup_error`.
    #[serde(rename = "type")]
    pub kind: &'static str,
    /// Upstream puts a label here, such as `"observation_value"`, not the input.
    pub input: &'static str,
}

impl RouteError {
    /// A `value_error` at `["body"]`.
    #[must_use]
    pub fn value_error(msg: impl Into<String>, input: &'static str) -> Self {
        Self {
            loc: vec!["body".into()],
            msg: msg.into(),
            kind: "value_error",
            input,
        }
    }
}

/// `{"detail": [...]}`, the 422 body.
#[derive(Debug, Serialize)]
pub struct Detail<T> {
    /// The errors.
    pub detail: T,
}

/// An inline bulk error, re-validated upstream as `APIErrorDetail`:
/// key order `type`, `loc`, `msg`, `input`, `ctx`, `url`.
#[derive(Debug, Serialize)]
pub struct InlineError {
    #[serde(rename = "type")]
    kind: &'static str,
    loc: Vec<LocItem>,
    msg: String,
    input: &'static str,
    ctx: Option<Value>,
    url: Option<String>,
}

impl From<RouteError> for InlineError {
    fn from(error: RouteError) -> Self {
        Self {
            kind: error.kind,
            loc: error.loc,
            msg: error.msg,
            input: error.input,
            ctx: None,
            url: None,
        }
    }
}

/// Pydantic request-validation errors, as FastAPI returns them.
pub type ValidationErrors = Detail<Vec<FieldError>>;
