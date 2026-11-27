// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Benchmark round 0 (spec/benchmark.md): the engine in-process.
//!
//! For every R-vector child it times the UK-WHO SDS at chronological and
//! corrected age (`sds_for_measurement`), and the whole calculation
//! (`measure`), with ages and dates prepared beforehand, as
//! bench/round0/python.py does for rcpchgrowth. Each figure is the best of
//! several passes. `--json` prints them as JSON, for s/bench.
//!
//! Run: s/bench-round0, which pins this and its Python counterpart to the
//! same CPU.

use std::hint::black_box;
use std::time::Instant;

use jiff::civil::Date;
#[path = "../tests/common/mod.rs"]
mod common;

use common::{DATA_DIR_ENV, read_json};
use opengrowth::dates::{chronological_decimal_age, corrected_decimal_age};
use opengrowth::store::ReferenceStore;
use opengrowth::{
    Gestation, GrowthReference, MeasurementInput, MeasurementMethod, Sex, measure,
    sds_for_measurement,
};

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

/// The best time per item, in nanoseconds, over `passes` runs of `work`,
/// each covering `items` items.
fn best_of(passes: usize, items: usize, mut work: impl FnMut()) -> f64 {
    let mut best = f64::INFINITY;
    for _ in 0..passes {
        let start = Instant::now();
        work();
        best = best.min(start.elapsed().as_nanos() as f64 / items as f64);
    }
    best
}

fn main() {
    let json = std::env::args().any(|arg| arg == "--json");
    let dir = std::env::var_os(DATA_DIR_ENV).map(std::path::PathBuf::from);
    let store = ReferenceStore::standard(dir.as_deref()).unwrap();
    let vectors: Vec<RVector> = read_json("independent/uk-who-cole-r-vectors.json");
    let children: Vec<MeasurementInput> = vectors
        .iter()
        .map(|v| MeasurementInput {
            reference: GrowthReference::UkWho,
            birth_date: parse_date(&v.birth_date),
            observation_date: parse_date(&v.observation_date),
            gestation: Gestation::new(v.gestation_weeks, v.gestation_days).unwrap(),
            sex: v.sex,
            method: v.measurement_method,
            observation_value: v.observation_value,
        })
        .collect();
    let sds_inputs: Vec<(f64, MeasurementMethod, f64, Sex)> = children
        .iter()
        .flat_map(|child| {
            [
                chronological_decimal_age(child.birth_date, child.observation_date),
                corrected_decimal_age(child.birth_date, child.observation_date, child.gestation)
                    .unwrap(),
            ]
            .map(|age| (age, child.method, child.observation_value, child.sex))
        })
        .collect();

    let rounds = 200;
    let sds_ns = best_of(5, rounds * sds_inputs.len(), || {
        for _ in 0..rounds {
            for &(age, method, value, sex) in &sds_inputs {
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
    });
    let rounds = 20;
    let measurement_ns = best_of(5, rounds * children.len(), || {
        for _ in 0..rounds {
            for child in &children {
                let _ = black_box(measure(&store, child));
            }
        }
    });

    if json {
        let result = serde_json::json!({
            "engine": "opengrowth",
            "version": opengrowth::ENGINE_VERSION,
            "children": children.len(),
            "sds_ns": sds_ns,
            "measurement_ns": measurement_ns,
        });
        println!("{result}");
    } else {
        println!(
            "rust   sds_for_measurement: {sds_ns:>10.1} ns per SDS ({:.0} per second, 1 core)",
            1e9 / sds_ns
        );
        println!(
            "rust   measure (full):      {measurement_ns:>10.1} ns per measurement, 2 SDS + extras ({:.0} per second, 1 core)",
            1e9 / measurement_ns
        );
    }
}
