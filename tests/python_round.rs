// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

//! `python_round` must agree with CPython `round(x, n)` to the bit, and
//! `numpy_round` and `python_round_int` with `round()` on a NumPy `float64`.

mod common;

use common::{Golden, read_json};
use opengrowth::round::{numpy_round, python_round, python_round_int};

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

#[derive(serde::Deserialize)]
struct NumPyCase {
    x: f64,
    /// `None` for `round(x)` with no digits.
    ndigits: Option<usize>,
    rounded: f64,
}

#[test]
fn numpy_round_matches_numpy_bit_for_bit() {
    let golden: Golden<Vec<NumPyCase>> = read_json("expected/numpy-round.json");
    let ours = |c: &NumPyCase| {
        c.ndigits
            .map_or_else(|| python_round_int(c.x), |n| numpy_round(c.x, n))
    };
    let mismatches: Vec<String> = golden
        .cases
        .iter()
        .filter(|c| ours(c).to_bits() != c.rounded.to_bits())
        .map(|c| {
            format!(
                "round(float64({:?}), {:?}) = {:?}, got {:?}",
                c.x,
                c.ndigits,
                c.rounded,
                ours(c)
            )
        })
        .collect();
    assert!(
        mismatches.is_empty(),
        "{} mismatches:\n{}",
        mismatches.len(),
        mismatches.join("\n")
    );
    let differ = golden
        .cases
        .iter()
        .filter(|c| {
            c.ndigits
                .is_some_and(|n| python_round(c.x, n).to_bits() != c.rounded.to_bits())
        })
        .count();
    eprintln!(
        "numpy_round: {} cases bit-exact; CPython rounding would differ in {differ}",
        golden.cases.len()
    );
}
