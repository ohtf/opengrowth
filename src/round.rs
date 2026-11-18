// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Python-compatible rounding.
//!
//! Python's `round(x, n)` rounds the *exact* binary value of `x` to `n`
//! decimal places, breaking exact ties to even, and converts back to the
//! nearest `f64`. Rust's `f64::round` rounds half away from zero and has no
//! digits argument, so it must never reach an output field (agent
//! instructions, core invariants).
//!
//! Rust's fixed-precision float formatting is exact and rounds ties to even,
//! and `str::parse::<f64>` is correctly rounded, which together reproduce
//! CPython's `double_round`. The conformance suite checks this against values
//! captured from CPython, including the classic `round(2.675, 2) == 2.67`.

use std::fmt::Write;

/// Rounds like Python's `round(x, ndigits)` for `ndigits` in `0..=17`.
///
/// Non-finite values are returned unchanged, as in Python.
#[must_use]
pub fn python_round(x: f64, ndigits: usize) -> f64 {
    debug_assert!(ndigits <= 17, "python_round supports up to 17 digits");
    if !x.is_finite() {
        return x;
    }
    let mut buffer = String::with_capacity(32);
    write!(buffer, "{x:.ndigits$}").expect("writing to a String cannot fail");
    buffer
        .parse()
        .expect("a formatted finite f64 always parses back")
}

#[cfg(test)]
mod tests {
    use super::python_round;

    #[test]
    fn matches_known_cpython_results() {
        // Each expected value is what CPython 3.13 returns.
        let cases = [
            (2.675, 2, 2.67),        // 2.675 is stored just below 2.675
            (0.125, 2, 0.12),        // exact tie, rounds to even
            (0.375, 2, 0.38),        // exact tie, rounds to even
            (2.5, 0, 2.0),           // exact tie, rounds to even
            (3.5, 0, 4.0),           // exact tie, rounds to even
            (-0.5, 0, -0.0),         // keeps the sign of zero
            (1.000_05, 4, 1.0001),   // stored just above the tie
            (12.345_65, 4, 12.3456), // stored just below the tie
            (0.038_329_911, 4, 0.0383),
        ];
        for (value, digits, expected) in cases {
            let actual = python_round(value, digits);
            assert_eq!(
                actual.to_bits(),
                f64::to_bits(expected),
                "round({value}, {digits}) gave {actual}, expected {expected}"
            );
        }
    }

    #[test]
    fn rounding_differs_from_f64_round_at_ties() {
        assert_eq!(2.5_f64.round(), 3.0);
        assert_eq!(python_round(2.5, 0), 2.0);
    }

    #[test]
    fn non_finite_values_pass_through() {
        assert!(python_round(f64::NAN, 4).is_nan());
        assert_eq!(python_round(f64::INFINITY, 4), f64::INFINITY);
    }
}
