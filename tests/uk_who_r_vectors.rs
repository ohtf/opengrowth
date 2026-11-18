// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

#![cfg(feature = "embedded-data")]

//! UK-WHO SDS against both oracles (`spec/standard.md` sections 5.2 and 8):
//!
//! 1. The independent R vectors: about 4,000 fictional children computed by
//!    Tim Cole's R implementation, at 1e-3 absolute SDS, the gate the Python
//!    engine passes.
//! 2. The Python reference engine's output for the same inputs, at 1e-9,
//!    with the bit-exact agreement rate reported because a byte-identical
//!    API (Profile B) needs bit-identical SDS.
//!
//! Uses the embedded tables, or the files in `$OPENGROWTH_REFERENCE_DATA_DIR`
//! layered over them if it is set.

mod common;

use common::{Agreement, DATA_DIR_ENV, Golden, read_json};
use jiff::civil::Date;
use opengrowth::dates::{chronological_decimal_age, corrected_decimal_age};
use opengrowth::lms::Lms;
use opengrowth::store::ReferenceStore;
use opengrowth::{
    Gestation, GrowthReference, MeasurementMethod, SdsResult, Sex, sds_for_measurement,
};

/// The independent oracle's tolerance, as in rcpchgrowth's test_uk_who.py.
const R_TOLERANCE: f64 = 1e-3;
/// The Python goldens' tolerance for SDS (`spec/standard.md` section 5.2).
const PYTHON_TOLERANCE: f64 = 1e-9;

#[derive(serde::Deserialize)]
struct RVector {
    birth_date: String,
    observation_date: String,
    gestation_weeks: u8,
    gestation_days: u8,
    sex: Sex,
    measurement_method: MeasurementMethod,
    observation_value: f64,
    corrected_sds: Option<f64>,
    chronological_sds: Option<f64>,
}

#[derive(serde::Deserialize)]
struct PythonCase {
    index: usize,
    chronological_decimal_age: f64,
    corrected_decimal_age: f64,
    chronological_sds: Option<f64>,
    corrected_sds: Option<f64>,
    chronological_measurement_error: Option<String>,
    corrected_measurement_error: Option<String>,
}

fn parse_date(text: &str) -> Date {
    let mut parts = text
        .split('/')
        .map(|p| p.parse::<i16>().expect("numeric date part"));
    let (day, month, year) = (
        parts.next().unwrap(),
        parts.next().unwrap(),
        parts.next().unwrap(),
    );
    Date::new(year, month as i8, day as i8).expect("valid date")
}

/// The same SDS computed with the `libm` crate's `pow`, to measure whether a
/// platform-independent `pow` would track Python better than `std`'s.
fn sds_with_libm_pow(observation: f64, lms: &Lms) -> f64 {
    if lms.l != 0.0 {
        (libm::pow(observation / lms.m, lms.l) - 1.0) / (lms.l * lms.s)
    } else {
        libm::log(observation / lms.m) / lms.s
    }
}

#[test]
fn uk_who_sds_matches_r_vectors_and_python() {
    let dir = std::env::var_os(DATA_DIR_ENV).map(std::path::PathBuf::from);
    let store = ReferenceStore::standard(dir.as_deref()).expect("reference tables load");
    assert_eq!(store.loaded().len(), 4, "all four UK-WHO tables are needed");

    let vectors: Vec<RVector> = read_json("independent/uk-who-cole-r-vectors.json");
    let golden: Golden<Vec<PythonCase>> = read_json("expected/uk-who-r-vector-inputs.json");
    assert_eq!(vectors.len(), golden.cases.len());
    eprintln!("Python reference environment: {}", golden.environment);

    let mut failures = Vec::new();
    let mut vs_r = Agreement::default();
    let mut vs_python = Agreement::default();
    let mut vs_python_libm_pow = Agreement::default();
    let mut ages = Agreement::default();
    let mut both_absent = 0;

    for (vector, python) in vectors.iter().zip(&golden.cases) {
        let i = python.index;
        let birth = parse_date(&vector.birth_date);
        let observed = parse_date(&vector.observation_date);
        let gestation =
            Gestation::new(vector.gestation_weeks, vector.gestation_days).expect("gestation");
        let chronological_age = chronological_decimal_age(birth, observed);
        let corrected_age =
            corrected_decimal_age(birth, observed, gestation).expect("dates in order");
        ages.record(chronological_age, python.chronological_decimal_age);
        ages.record(corrected_age, python.corrected_decimal_age);

        let calculate = |age| {
            sds_for_measurement(
                &store,
                GrowthReference::UkWho,
                age,
                vector.measurement_method,
                vector.observation_value,
                vector.sex,
            )
        };
        let kinds = [
            (
                "corrected",
                calculate(corrected_age),
                vector.corrected_sds,
                python.corrected_sds,
                &python.corrected_measurement_error,
            ),
            (
                "chronological",
                calculate(chronological_age),
                vector.chronological_sds,
                python.chronological_sds,
                &python.chronological_measurement_error,
            ),
        ];
        for (kind, rust, r, py, py_error) in kinds {
            let rust_sds = rust.as_ref().ok().map(|r: &SdsResult| r.sds);
            let label = format!(
                "case {i} {kind} ({} {} {})",
                vector.sex, vector.measurement_method, vector.birth_date
            );

            match (rust_sds, r) {
                (Some(ours), Some(theirs)) => {
                    vs_r.record(ours, theirs);
                    if (ours - theirs).abs() > R_TOLERANCE {
                        failures.push(format!("{label}: R {theirs}, Rust {ours}"));
                    }
                }
                (None, None) => both_absent += 1,
                (ours, theirs) => failures.push(format!(
                    "{label}: R {theirs:?}, Rust {ours:?} ({:?})",
                    rust.as_ref().err()
                )),
            }

            match (&rust, py) {
                (Ok(result), Some(theirs)) => {
                    vs_python.record(result.sds, theirs);
                    vs_python_libm_pow.record(
                        sds_with_libm_pow(vector.observation_value, &result.lms),
                        theirs,
                    );
                    if (result.sds - theirs).abs() > PYTHON_TOLERANCE {
                        failures.push(format!("{label}: Python {theirs}, Rust {}", result.sds));
                    }
                }
                (Err(error), None) => {
                    if py_error.as_deref() != Some(error.to_string().as_str()) {
                        failures.push(format!(
                            "{label}: Python error {py_error:?}, Rust error {error}"
                        ));
                    }
                }
                (rust, theirs) => {
                    failures.push(format!("{label}: Python {theirs:?}, Rust {rust:?}"))
                }
            }
        }
    }

    eprintln!("ages vs Python:            {}", ages.summary());
    eprintln!("SDS vs R (tolerance 1e-3): {}", vs_r.summary());
    eprintln!("SDS vs Python (std pow):   {}", vs_python.summary());
    eprintln!(
        "SDS vs Python (libm pow):  {}",
        vs_python_libm_pow.summary()
    );
    eprintln!("absent in both R and Rust: {both_absent}");
    assert!(
        failures.is_empty(),
        "{} failures:\n{}",
        failures.len(),
        failures
            .iter()
            .take(50)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}
