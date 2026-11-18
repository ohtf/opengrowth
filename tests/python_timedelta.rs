// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

//! `python_add_days` must agree with CPython `date + timedelta(days=x)`,
//! including where CPython raises.

mod common;

use common::{Golden, read_json};
use jiff::civil::date;
use opengrowth::dates::python_add_days;

#[derive(serde::Deserialize)]
struct Case {
    days: f64,
    result: Option<String>,
}

#[test]
fn python_add_days_matches_cpython() {
    let golden: Golden<Vec<Case>> = read_json("expected/python-timedelta.json");
    let birth = date(1759, 4, 11);
    let mismatches: Vec<String> = golden
        .cases
        .iter()
        .filter_map(|c| {
            let ours = python_add_days(birth, c.days).map(|d| d.to_string());
            (ours != c.result)
                .then(|| format!("days={:?}: CPython {:?}, ours {ours:?}", c.days, c.result))
        })
        .collect();
    assert!(
        mismatches.is_empty(),
        "{} mismatches:\n{}",
        mismatches.len(),
        mismatches.join("\n")
    );
    eprintln!("python_add_days: {} cases identical", golden.cases.len());
}
