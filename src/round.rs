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
//!
//! A NumPy `float64`, which is what SciPy functions return, has its own
//! `round` ([`numpy_round`]), and Python's `round()` calls it instead.
//! Upstream values are one or the other depending on where they came from,
//! so code that reproduces upstream rounding carries the kind with the value
//! ([`PyFloat`]).

use std::fmt::Write;

/// A float as the upstream engine holds it: a Python `float` or a NumPy
/// `float64`. The value and its printed form are the same either way; only
/// rounding differs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PyFloat {
    /// A Python `float`.
    Builtin(f64),
    /// A NumPy `float64`, as returned by SciPy, or by arithmetic with one.
    NumPy(f64),
}

impl PyFloat {
    /// The value.
    #[must_use]
    pub const fn value(self) -> f64 {
        match self {
            Self::Builtin(x) | Self::NumPy(x) => x,
        }
    }

    /// Arithmetic with a Python float, which keeps the kind: a NumPy float
    /// stays one.
    #[must_use]
    pub fn map(self, f: impl FnOnce(f64) -> f64) -> Self {
        match self {
            Self::Builtin(x) => Self::Builtin(f(x)),
            Self::NumPy(x) => Self::NumPy(f(x)),
        }
    }

    /// Python's `round(self, ndigits)`, which keeps the kind.
    #[must_use]
    pub fn round(self, ndigits: usize) -> Self {
        match self {
            Self::Builtin(x) => Self::Builtin(python_round(x, ndigits)),
            Self::NumPy(x) => Self::NumPy(numpy_round(x, ndigits)),
        }
    }
}

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

/// Rounds like Python's `round(x, decimals)` on a NumPy `float64`, for
/// `decimals` in `0..=8`: scale by a power of ten, round half to even, and
/// scale back, as NumPy's `round` does.
///
/// This differs from [`python_round`] where the scaling lands exactly on a
/// tie that the exact value of `x` is not on. `42.34725` is stored just above
/// the tie, so a Python float rounds to `42.3473` at four places; but
/// `42.34725 * 10000` is exactly `423472.5` in floating point, so a `float64`
/// rounds to `42.3472`. The sign of zero is kept, as NumPy keeps it.
#[must_use]
pub fn numpy_round(x: f64, decimals: usize) -> f64 {
    const POWERS_OF_TEN: [f64; 9] = [1e0, 1e1, 1e2, 1e3, 1e4, 1e5, 1e6, 1e7, 1e8];
    debug_assert!(decimals <= 8, "numpy_round supports up to 8 decimals");
    let scale = POWERS_OF_TEN[decimals];
    (x * scale).round_ties_even() / scale
}

/// Python's `round(x)` with no `ndigits`, for a Python float or a NumPy
/// `float64` alike: the nearest integer, ties to even. Python returns an
/// `int`, which has no negative zero, so neither does this.
///
/// Only for finite `x`; Python raises for infinity and NaN.
#[must_use]
pub fn python_round_int(x: f64) -> f64 {
    let rounded = x.round_ties_even();
    if rounded == 0.0 { 0.0 } else { rounded }
}

#[cfg(test)]
mod tests {
    use super::{PyFloat, numpy_round, python_round, python_round_int};

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
        assert!(numpy_round(f64::NAN, 4).is_nan());
        assert_eq!(numpy_round(f64::NEG_INFINITY, 4), f64::NEG_INFINITY);
    }

    #[test]
    fn numpy_and_python_rounding_differ_at_a_scaled_tie() {
        // Both expected values are what Python 3.13 with NumPy 2.5 returns.
        assert_eq!(python_round(42.347_25, 4), 42.3473);
        assert_eq!(numpy_round(42.347_25, 4), 42.3472);
        assert_eq!(PyFloat::NumPy(42.347_25).round(4), PyFloat::NumPy(42.3472));
        assert_eq!(
            PyFloat::Builtin(42.347_25).round(4),
            PyFloat::Builtin(42.3473)
        );
    }

    #[test]
    fn rounding_to_an_integer_never_gives_negative_zero() {
        assert_eq!(python_round_int(-0.25).to_bits(), 0.0_f64.to_bits());
        assert_eq!(python_round_int(-2.5), -2.0);
        assert_eq!(python_round_int(3.5), 4.0);
    }
}
