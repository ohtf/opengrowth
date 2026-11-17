// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

//! `python_repr` must spell every float exactly as CPython `repr` does (R31).

use opengrowth_conformance::{Golden, read_json};
use opengrowth_core::pyfmt::python_repr;

#[derive(serde::Deserialize)]
struct Case {
    bits: String,
    repr: String,
}

fn decode(bits: &str) -> f64 {
    let mut bytes = [0u8; 8];
    for (i, byte) in bytes.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&bits[2 * i..2 * i + 2], 16).expect("hex");
    }
    f64::from_le_bytes(bytes)
}

#[test]
fn python_repr_matches_cpython() {
    let golden: Golden<Vec<Case>> = read_json("expected/python-repr.json");
    let mismatches: Vec<String> = golden
        .cases
        .iter()
        .filter_map(|c| {
            let ours = python_repr(decode(&c.bits));
            (ours != c.repr).then(|| format!("{}: CPython {}, ours {ours}", c.bits, c.repr))
        })
        .collect();
    assert!(
        mismatches.is_empty(),
        "{} mismatches:\n{}",
        mismatches.len(),
        mismatches
            .iter()
            .take(40)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
    eprintln!("python_repr: {} cases identical", golden.cases.len());
}
