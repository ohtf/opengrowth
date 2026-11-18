// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

#![cfg(feature = "embedded-data")]

//! The engine conformance cases (`conformance/expected/engine-*.json`):
//! rcpchgrowth 4.6.5's own values at every boundary each reference defines,
//! compared with opengrowth's engine directly, not through the HTTP API.
//! Every value must be identical, floats to the last bit.

mod common;

use std::collections::BTreeMap;

use common::{DATA_DIR_ENV, Golden, read_json};
use jiff::civil::Date;
use opengrowth::advice::{CentileFormat, centile_band};
use opengrowth::store::{ReferenceStore, manifest};
use opengrowth::{
    Gestation, GrowthReference, MeasureError, Measurement, MeasurementInput, MeasurementMethod,
    Sex, centile, measure, sds_for_measurement,
};
use serde_json::{Value, json};

const REFERENCES: [&str; 6] = [
    "uk-who",
    "who",
    "cdc",
    "turners-syndrome",
    "trisomy-21",
    "trisomy-21-aap",
];

#[derive(serde::Deserialize)]
struct SdsInput {
    reference: GrowthReference,
    measurement_method: MeasurementMethod,
    sex: Sex,
    age: f64,
    observation_value: f64,
}

#[derive(Debug, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct SdsExpected {
    sds: Option<f64>,
    centile: Option<f64>,
    table: Option<String>,
    error: Option<String>,
}

#[derive(serde::Deserialize)]
struct SdsCase {
    name: String,
    input: SdsInput,
    expected: SdsExpected,
}

#[derive(serde::Deserialize)]
struct MeasurementCaseInput {
    reference: GrowthReference,
    birth_date: String,
    observation_date: String,
    gestation_weeks: u8,
    gestation_days: u8,
    sex: Sex,
    measurement_method: MeasurementMethod,
    observation_value: f64,
}

#[derive(serde::Deserialize)]
struct MeasurementCase {
    name: String,
    purpose: String,
    input: MeasurementCaseInput,
    expected: Value,
}

/// Where opengrowth's engine deliberately differs from rcpchgrowth's
/// (`spec/standard.md` section 6.3): the cases with this purpose, the field,
/// and why. Each must still differ in exactly this way, and the rest of each
/// case is compared as usual.
const KNOWN_DIFFERENCES: &[(&str, [&str; 2], &str)] = &[(
    "observation before birth",
    ["child_observation_value", "observation_value_error"],
    "with no corrected age, upstream's observation check compares None with its SDS limit and \
     records the Python TypeError text; opengrowth records no error. The API rejects such a \
     request before calculating, so no client sees it",
)];

/// Upstream's text for the known difference.
const LEAKED_TYPE_ERROR: &str = "'<' not supported between instances of 'NoneType' and ";

#[derive(serde::Deserialize)]
struct ReferenceCases {
    sds: Vec<SdsCase>,
    measurement: Vec<MeasurementCase>,
}

#[derive(serde::Deserialize)]
struct BandInput {
    sds: f64,
    measurement_method: MeasurementMethod,
    centile_format: String,
}

#[derive(serde::Deserialize)]
struct BandCase {
    name: String,
    input: BandInput,
    expected: BTreeMap<String, Option<String>>,
}

fn store() -> ReferenceStore {
    let dir = std::env::var_os(DATA_DIR_ENV).map(std::path::PathBuf::from);
    ReferenceStore::standard(dir.as_deref()).expect("tables load")
}

/// opengrowth's result for an SDS case, in the case's terms.
fn sds_outcome(store: &ReferenceStore, input: &SdsInput) -> SdsExpected {
    match sds_for_measurement(
        store,
        input.reference,
        input.age,
        input.measurement_method,
        input.observation_value,
        input.sex,
    ) {
        Ok(result) => SdsExpected {
            sds: Some(result.sds),
            centile: Some(centile(result.sds)),
            table: Some(manifest::source(result.table).file.to_owned()),
            error: None,
        },
        Err(error) => SdsExpected {
            sds: None,
            centile: None,
            table: None,
            error: Some(error.to_string()),
        },
    }
}

/// The data file each SDS came from, as the generator records it.
fn table_at(
    store: &ReferenceStore,
    input: &MeasurementInput,
    age: Option<f64>,
    sds: Option<f64>,
) -> Value {
    let (Some(age), Some(_)) = (age, sds) else {
        return Value::Null;
    };
    let result = sds_for_measurement(
        store,
        input.reference,
        age,
        input.method,
        input.observation_value,
        input.sex,
    )
    .expect("an SDS was calculated at this age");
    json!(manifest::source(result.table).file)
}

/// opengrowth's measurement in the shape of upstream's `Measurement.measurement`,
/// without the plotting and bone-age echo sections. Unrounded: the API's
/// rounding of four centile fields happens in the server, as upstream's does.
fn project(store: &ReferenceStore, input: &MeasurementInput, m: &Measurement) -> Value {
    let birth = &m.birth_data;
    let dates = &m.measurement_dates;
    let values = &m.values;
    let comments = dates.comments.as_ref();
    let (weeks, days) = dates
        .corrected_gestational_age
        .map_or((None, None), |(w, d)| (Some(w), Some(d)));
    json!({
        "birth_data": {
            "birth_date": birth.birth_date.to_string(),
            "gestation_weeks": birth.gestation.weeks,
            "gestation_days": birth.gestation.days,
            "estimated_date_delivery": birth.estimated_date_delivery.map(|d| d.to_string()),
            "estimated_date_delivery_string": birth.estimated_date_delivery_string,
            "sex": birth.sex.as_str(),
        },
        "measurement_dates": {
            "observation_date": dates.observation_date.to_string(),
            "chronological_decimal_age": dates.chronological_decimal_age,
            "corrected_decimal_age": dates.corrected_decimal_age,
            "chronological_calendar_age": dates.chronological_calendar_age,
            "corrected_calendar_age": dates.corrected_calendar_age,
            "corrected_gestational_age": {
                "corrected_gestation_weeks": weeks,
                "corrected_gestation_days": days,
            },
            "comments": {
                "clinician_corrected_decimal_age_comment": comments.map(|c| &c.clinician_corrected),
                "lay_corrected_decimal_age_comment": comments.map(|c| &c.lay_corrected),
                "clinician_chronological_decimal_age_comment": comments.map(|c| &c.clinician_chronological),
                "lay_chronological_decimal_age_comment": comments.map(|c| &c.lay_chronological),
            },
            "corrected_decimal_age_error": dates.corrected_decimal_age_error,
            "chronological_decimal_age_error": dates.chronological_decimal_age_error,
        },
        "child_observation_value": {
            "measurement_method": m.method.as_str(),
            "observation_value": m.observation_value,
            "observation_value_error": m.observation_value_error,
        },
        "measurement_calculated_values": {
            "corrected_sds": values.corrected_sds,
            "corrected_centile": values.corrected_centile,
            "corrected_centile_band": values.corrected_centile_band,
            "chronological_sds": values.chronological_sds,
            "chronological_centile": values.chronological_centile,
            "chronological_centile_band": values.chronological_centile_band,
            "corrected_measurement_error": values.corrected_measurement_error,
            "chronological_measurement_error": values.chronological_measurement_error,
            "corrected_percentage_median_bmi": values.corrected_percentage_median_bmi,
            "chronological_percentage_median_bmi": values.chronological_percentage_median_bmi,
        },
        "tables": {
            "chronological": table_at(store, input, Some(dates.chronological_decimal_age), values.chronological_sds),
            "corrected": table_at(store, input, dates.corrected_decimal_age, values.corrected_sds),
        },
    })
}

/// The first place two JSON values differ, as a path and both values.
fn first_difference(path: &str, expected: &Value, actual: &Value) -> Option<String> {
    match (expected, actual) {
        (Value::Object(e), Value::Object(a)) => {
            for key in e.keys().chain(a.keys()) {
                let (ev, av) = (
                    e.get(key).unwrap_or(&Value::Null),
                    a.get(key).unwrap_or(&Value::Null),
                );
                if !e.contains_key(key) || !a.contains_key(key) {
                    return Some(format!("{path}.{key}: present in only one"));
                }
                if let Some(found) = first_difference(&format!("{path}.{key}"), ev, av) {
                    return Some(found);
                }
            }
            None
        }
        _ if expected == actual => None,
        _ => Some(format!("{path}: expected {expected}, got {actual}")),
    }
}

/// Accounts for a known difference: checks it still holds, then compares the
/// rest of the case as if upstream had matched. Returns the expected value to
/// compare, or why the difference no longer holds.
fn allow_known_difference(case: &MeasurementCase, actual: &Value) -> Result<Value, String> {
    let mut expected = case.expected.clone();
    for (purpose, [section, field], reason) in KNOWN_DIFFERENCES {
        if case.purpose != *purpose {
            continue;
        }
        let upstream = expected[section][field].as_str().unwrap_or("");
        if !upstream.starts_with(LEAKED_TYPE_ERROR) || !actual[section][field].is_null() {
            return Err(format!(
                "listed as a known difference ({reason}), but upstream has {:?} and opengrowth {}",
                upstream, actual[section][field]
            ));
        }
        expected[section][field] = Value::Null;
    }
    Ok(expected)
}

fn measurement_failure(store: &ReferenceStore, case: &MeasurementCase) -> Option<String> {
    let i = &case.input;
    let input = MeasurementInput {
        reference: i.reference,
        birth_date: i.birth_date.parse::<Date>().expect("ISO date"),
        observation_date: i.observation_date.parse::<Date>().expect("ISO date"),
        gestation: Gestation::new(i.gestation_weeks, i.gestation_days).expect("valid gestation"),
        sex: i.sex,
        method: i.measurement_method,
        observation_value: i.observation_value,
    };
    let actual = match measure(store, &input) {
        Ok(m) => project(store, &input, &m),
        Err(MeasureError::Raised(message)) => json!({ "error": message }),
        Err(error) => return Some(format!("{error}")),
    };
    match allow_known_difference(case, &actual) {
        Ok(expected) => first_difference("", &expected, &actual),
        Err(why) => Some(why),
    }
}

#[test]
fn engine_matches_rcpchgrowth_at_every_boundary() {
    let store = store();
    let mut failures = Vec::new();
    let mut known = 0;
    for reference in REFERENCES {
        let golden: Golden<ReferenceCases> =
            read_json(&format!("expected/engine-{reference}.json"));
        let (sds, measurements) = (&golden.cases.sds, &golden.cases.measurement);
        let before = failures.len();
        for case in sds {
            let actual = sds_outcome(&store, &case.input);
            if actual != case.expected {
                failures.push(format!(
                    "{reference} sds {}: expected {:?}, got {actual:?}",
                    case.name, case.expected
                ));
            }
        }
        known += measurements
            .iter()
            .filter(|c| KNOWN_DIFFERENCES.iter().any(|(p, _, _)| c.purpose == *p))
            .count();
        for case in measurements {
            if let Some(found) = measurement_failure(&store, case) {
                failures.push(format!("{reference} measurement {}: {found}", case.name));
            }
        }
        eprintln!(
            "{reference}: {} SDS and {} measurement cases, {} differ",
            sds.len(),
            measurements.len(),
            failures.len() - before
        );
    }
    eprintln!("{known} cases with a known difference, each checked to still hold");
    assert!(
        known >= KNOWN_DIFFERENCES.len(),
        "a known difference has no case"
    );
    let shown: Vec<_> = failures.iter().take(20).cloned().collect();
    assert!(
        failures.is_empty(),
        "{} cases differ:\n{}",
        failures.len(),
        shown.join("\n")
    );
}

#[test]
fn centile_bands_match_rcpchgrowth_at_every_edge() {
    let golden: Golden<Vec<BandCase>> = read_json("expected/engine-centile-bands.json");
    let mut failures = Vec::new();
    for case in &golden.cases {
        let format = CentileFormat::from_name(&case.input.centile_format).expect("a known format");
        let actual = centile_band(case.input.sds, case.input.measurement_method, format);
        let expected = case.expected.get("centile_band").expect("centile_band");
        if &actual != expected {
            failures.push(format!(
                "{}: expected {expected:?}, got {actual:?}",
                case.name
            ));
        }
    }
    eprintln!(
        "{} centile band cases, {} differ",
        golden.cases.len(),
        failures.len()
    );
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
