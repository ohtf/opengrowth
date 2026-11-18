// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

//! `python_round` must agree with CPython `round(x, n)` to the bit.

mod common;

use common::{Golden, read_json};
use opengrowth::round::python_round;

#[derive(serde::Deserialize)]
struct Case {
    x: f64,
    ndigits: usize,
    rounded: f64,
}

#[test]
fn python_round_matches_cpython_bit_for_bit() {
    let golden: Golden<Vec<Case>> = read_json("expected/python-round.json");
    let mismatches: Vec<String> = golden
        .cases
        .iter()
        .filter(|c| python_round(c.x, c.ndigits).to_bits() != c.rounded.to_bits())
        .map(|c| {
            format!(
                "round({:?}, {}) = {:?}, got {:?}",
                c.x,
                c.ndigits,
                c.rounded,
                python_round(c.x, c.ndigits)
            )
        })
        .collect();
    assert!(
        mismatches.is_empty(),
        "{} mismatches:\n{}",
        mismatches.len(),
        mismatches.join("\n")
    );
    eprintln!("python_round: {} cases bit-exact", golden.cases.len());
}
