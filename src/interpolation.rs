// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-FileCopyrightText: Royal College of Paediatrics and Child Health
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Interpolation of LMS parameters between tabulated ages.
//!
//! Both functions reproduce the upstream engine's arithmetic exactly:
//!
//! - [`cubic_lagrange`] is the four-point Lagrange polynomial from Tim Cole's
//!   LMSgrowth, as transcribed in `rcpchgrowth.global_functions.cubic_interpolation`.
//!   It is reimplemented here from that published method, not copied from
//!   LMSgrowth's source (see `spec/standard.md` section 3.4).
//! - [`linear`] reproduces `scipy.interpolate.interp1d(kind="linear")`, which
//!   for one-dimensional `f64` data delegates to `numpy.interp`. Its formula
//!   is `slope * (x - x_lo) + y_lo`, which differs in the last bits from the
//!   de Boor form `interp1d` uses for other inputs.

/// Four-point Lagrange cubic interpolation.
///
/// `ages` and `values` are ordered two-below, one-below, one-above,
/// two-above. The operation order matches the upstream expression term for
/// term; do not refactor it into a loop or simplify it.
#[must_use]
pub fn cubic_lagrange(age: f64, ages: [f64; 4], values: [f64; 4]) -> f64 {
    let [age_two_below, age_one_below, age_one_above, age_two_above] = ages;
    let [
        value_two_below,
        value_one_below,
        value_one_above,
        value_two_above,
    ] = values;

    let tt0 = age - age_two_below;
    let tt1 = age - age_one_below;
    let tt2 = age - age_one_above;
    let tt3 = age - age_two_above;

    let t01 = age_two_below - age_one_below;
    let t02 = age_two_below - age_one_above;
    let t03 = age_two_below - age_two_above;
    let t12 = age_one_below - age_one_above;
    let t13 = age_one_below - age_two_above;
    let t23 = age_one_above - age_two_above;

    value_two_below * tt1 * tt2 * tt3 / t01 / t02 / t03
        - value_one_below * tt0 * tt2 * tt3 / t01 / t12 / t13
        + value_one_above * tt0 * tt1 * tt3 / t02 / t12 / t23
        - value_two_above * tt0 * tt1 * tt2 / t03 / t13 / t23
}

/// Linear interpolation between two points, as `numpy.interp` computes it.
///
/// The caller guarantees `x_lo <= x <= x_hi`; `interp1d` raises outside that
/// range, and the table lookup reports it as an error before calling this.
#[must_use]
pub fn linear(x: f64, x_lo: f64, x_hi: f64, y_lo: f64, y_hi: f64) -> f64 {
    if x >= x_hi {
        return y_hi;
    }
    let slope = (y_hi - y_lo) / (x_hi - x_lo);
    let result = slope * (x - x_lo) + y_lo;
    if !result.is_nan() {
        return result;
    }
    // numpy retries from the other end before giving up, which matters only
    // for infinite slopes.
    let retry = slope * (x - x_hi) + y_hi;
    if retry.is_nan() && y_lo == y_hi {
        y_lo
    } else {
        retry
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cubic_passes_through_its_nodes_and_reproduces_cubics() {
        let ages = [1.0, 2.0, 3.0, 4.0];
        let f = |x: f64| 2.0 * x * x * x - x * x + 0.5 * x - 7.0;
        let values = ages.map(f);
        for x in [1.5, 2.25, 2.5, 3.75] {
            assert!(
                (cubic_lagrange(x, ages, values) - f(x)).abs() < 1e-9,
                "x = {x}"
            );
        }
    }

    #[test]
    fn linear_hits_endpoints_and_midpoint() {
        assert_eq!(linear(0.0, 0.0, 2.0, 1.0, 3.0), 1.0);
        assert_eq!(linear(2.0, 0.0, 2.0, 1.0, 3.0), 3.0);
        assert_eq!(linear(1.0, 0.0, 2.0, 1.0, 3.0), 2.0);
    }
}
