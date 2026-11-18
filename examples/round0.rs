// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Benchmark round 0 (spec/benchmark.md): in-process engine throughput.
//!
//! Computes chronological and corrected UK-WHO SDS for every R-vector child,
//! repeatedly, and reports nanoseconds per SDS. Ages are precomputed so this
//! measures the same work as bench/round0/python.py's `sds_for_measurement` loop.
//!
//! Run: cargo run --release --example round0 (or s/bench-round0)

use std::hint::black_box;
use std::time::Instant;

use jiff::civil::Date;
#[path = "../tests/common/mod.rs"]
mod common;

use common::{DATA_DIR_ENV, read_json};
use opengrowth::dates::{chronological_decimal_age, corrected_decimal_age};
use opengrowth::store::ReferenceStore;
use opengrowth::{Gestation, GrowthReference, MeasurementMethod, Sex, sds_for_measurement};

#[derive(serde::Deserialize)]
struct RVector {
    birth_date: String,
    observation_date: String,
    gestation_weeks: u8,
    gestation_days: u8,
    sex: Sex,
    measurement_method: MeasurementMethod,
    observation_value: f64,
}

fn parse_date(text: &str) -> Date {
    let p: Vec<i16> = text.split('/').map(|x| x.parse().unwrap()).collect();
    Date::new(p[2], p[1] as i8, p[0] as i8).unwrap()
}

fn main() {
    let dir = std::env::var_os(DATA_DIR_ENV).map(std::path::PathBuf::from);
    let store = ReferenceStore::standard(dir.as_deref()).unwrap();
    let vectors: Vec<RVector> = read_json("independent/uk-who-cole-r-vectors.json");
    let inputs: Vec<(f64, MeasurementMethod, f64, Sex)> = vectors
        .iter()
        .flat_map(|v| {
            let birth = parse_date(&v.birth_date);
            let observed = parse_date(&v.observation_date);
            let gestation = Gestation::new(v.gestation_weeks, v.gestation_days).unwrap();
            [
                chronological_decimal_age(birth, observed),
                corrected_decimal_age(birth, observed, gestation).unwrap(),
            ]
            .map(|age| (age, v.measurement_method, v.observation_value, v.sex))
        })
        .collect();

    let rounds = 200;
    let mut best = f64::INFINITY;
    for _ in 0..5 {
        let start = Instant::now();
        for _ in 0..rounds {
            for &(age, method, value, sex) in &inputs {
                let _ = black_box(sds_for_measurement(
                    &store,
                    GrowthReference::UkWho,
                    age,
                    method,
                    value,
                    sex,
                ));
            }
        }
        let per_call = start.elapsed().as_nanos() as f64 / (rounds * inputs.len()) as f64;
        best = best.min(per_call);
    }
    println!(
        "rust  sds_for_measurement: {best:>10.1} ns per SDS ({:.0} per second, 1 core)",
        1e9 / best
    );
}
