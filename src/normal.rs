// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The standard normal distribution: centile from SDS and SDS from centile.
//!
//! The upstream engine uses `scipy.stats.norm.cdf` and `ppf`, which call
//! `scipy.special.ndtr` and `ndtri`. These are SciPy's own implementations,
//! translated line for line from SciPy's `xsf` library (`cephes`, BSD-3-Clause;
//! see `NOTICE.md`), so they agree with SciPy to the bit on the same platform.
//! The `normal_vs_scipy` conformance test checks that against a SciPy grid,
//! deep tails included (`spec/queries.md` Q5, `R32`).

mod cephes;

/// Cumulative probability of the standard normal distribution at `z`
/// (`scipy.special.ndtr`).
///
/// Multiply by 100 for a centile, as the upstream `centile()` does.
#[must_use]
pub fn cdf(z: f64) -> f64 {
    cephes::ndtr(z)
}

/// SDS for a cumulative probability `p`, the inverse of [`cdf`]
/// (`scipy.special.ndtri`).
///
/// Returns negative infinity at 0, positive infinity at 1, and NaN outside
/// `[0, 1]`, as SciPy does.
#[must_use]
pub fn ppf(p: f64) -> f64 {
    cephes::ndtri(p)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cdf_landmarks() {
        assert_eq!(cdf(0.0), 0.5);
        assert!((cdf(1.959_963_984_540_054) - 0.975).abs() < 1e-15);
        assert!((cdf(-1.0) - 0.158_655_253_931_457_05).abs() < 1e-16);
        assert_eq!(cdf(f64::INFINITY), 1.0);
        assert_eq!(cdf(f64::NEG_INFINITY), 0.0);
    }

    #[test]
    fn ppf_inverts_cdf_across_branches() {
        // Upper-tail round trips stop at z = 5: beyond that, cdf(z) is so close
        // to 1 that the f64 holds almost no information about z.
        for z in [-30.0, -8.0, -5.5, -3.0, -1.0, -0.2, 0.0, 0.7, 2.0, 4.5, 5.0] {
            let back = ppf(cdf(z));
            assert!(
                (back - z).abs() < 1e-9 * z.abs().max(1.0),
                "z = {z}, back = {back}"
            );
        }
    }

    #[test]
    fn ppf_edges() {
        assert_eq!(ppf(0.5), 0.0);
        assert_eq!(ppf(0.0), f64::NEG_INFINITY);
        assert_eq!(ppf(1.0), f64::INFINITY);
        assert!(ppf(-0.1).is_nan());
        assert!(ppf(1.1).is_nan());
    }
}
