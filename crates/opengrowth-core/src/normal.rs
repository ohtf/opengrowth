// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The standard normal distribution: centile from SDS and SDS from centile.
//!
//! The upstream engine uses `scipy.stats.norm.cdf` (`scipy.special.ndtr`) and
//! `scipy.stats.norm.ppf` (`scipy.special.ndtri`). Their exact agreement with
//! these implementations is measured against a SciPy-generated grid by the
//! `normal_vs_scipy` conformance test, and the results are recorded against
//! `Q4` and `Q5` in `spec/queries.md`.
//!
//! - [`cdf`] uses the structure of Cephes `ndtr`, which SciPy inherits, on top
//!   of the `libm` crate's `erf` and `erfc` (a port of musl, derived from
//!   FreeBSD's msun).
//! - [`ppf`] is algorithm AS241 `PPND16` (Wichura, 1988, Applied Statistics
//!   37:477-484), accurate to about 1 part in 10^16.

use std::f64::consts::FRAC_1_SQRT_2;

/// Cumulative probability of the standard normal distribution at `z`.
///
/// Multiply by 100 for a centile, as the upstream `centile()` does.
#[must_use]
pub fn cdf(z: f64) -> f64 {
    if z.is_nan() {
        return f64::NAN;
    }
    let x = z * FRAC_1_SQRT_2;
    let abs_x = x.abs();
    if abs_x < FRAC_1_SQRT_2 {
        0.5 + 0.5 * libm::erf(x)
    } else {
        let tail = 0.5 * libm::erfc(abs_x);
        if x > 0.0 { 1.0 - tail } else { tail }
    }
}

/// SDS for a cumulative probability `p` (the inverse of [`cdf`]).
///
/// Returns negative infinity at 0, positive infinity at 1, and NaN outside
/// `[0, 1]`, as SciPy does.
#[must_use]
pub fn ppf(p: f64) -> f64 {
    if p.is_nan() || !(0.0..=1.0).contains(&p) {
        return f64::NAN;
    }
    if p == 0.0 {
        return f64::NEG_INFINITY;
    }
    if p == 1.0 {
        return f64::INFINITY;
    }

    let q = p - 0.5;
    if q.abs() <= 0.425 {
        let r = 0.180_625 - q * q;
        return q * polynomial(r, &AS241_A) / polynomial_monic(r, &AS241_B);
    }

    let tail = if q < 0.0 { p } else { 1.0 - p };
    let mut r = (-tail.ln()).sqrt();
    let value = if r <= 5.0 {
        r -= 1.6;
        polynomial(r, &AS241_C) / polynomial_monic(r, &AS241_D)
    } else {
        r -= 5.0;
        polynomial(r, &AS241_E) / polynomial_monic(r, &AS241_F)
    };
    if q < 0.0 { -value } else { value }
}

/// `c[7] r^7 + ... + c[0]`, by Horner's rule from the highest power, as AS241
/// specifies.
fn polynomial(r: f64, c: &[f64; 8]) -> f64 {
    ((((((c[7] * r + c[6]) * r + c[5]) * r + c[4]) * r + c[3]) * r + c[2]) * r + c[1]) * r + c[0]
}

/// `c[7] r^7 + ... + c[1] r + 1`.
fn polynomial_monic(r: f64, c: &[f64; 8]) -> f64 {
    ((((((c[7] * r + c[6]) * r + c[5]) * r + c[4]) * r + c[3]) * r + c[2]) * r + c[1]) * r + 1.0
}

// AS241 coefficients. Index 0 of the monic denominators is unused.
#[expect(
    clippy::excessive_precision,
    reason = "published AS241 coefficients, transcribed verbatim"
)]
const AS241_A: [f64; 8] = [
    3.387_132_872_796_366_608,
    133.141_667_891_784_377_45,
    1_971.590_950_306_551_442_7,
    13_731.693_765_509_461_125,
    45_921.953_931_549_871_457,
    67_265.770_927_008_700_853,
    33_430.575_583_588_128_105,
    2_509.080_928_730_122_672_7,
];
#[expect(
    clippy::excessive_precision,
    reason = "published AS241 coefficients, transcribed verbatim"
)]
const AS241_B: [f64; 8] = [
    0.0,
    42.313_330_701_600_911_252,
    687.187_007_492_057_908_3,
    5_394.196_021_424_751_107_7,
    21_213.794_301_586_595_867,
    39_307.895_800_092_710_61,
    28_729.085_735_721_942_674,
    5_226.495_278_852_545_925,
];
#[expect(
    clippy::excessive_precision,
    reason = "published AS241 coefficients, transcribed verbatim"
)]
const AS241_C: [f64; 8] = [
    1.423_437_110_749_683_577_34,
    4.630_337_846_156_545_295_9,
    5.769_497_221_460_691_405_5,
    3.647_848_324_763_204_605_04,
    1.270_458_252_452_368_382_58,
    0.241_780_725_177_450_611_77,
    0.022_723_844_989_269_184_583_3,
    7.745_450_142_783_414_076_4e-4,
];
#[expect(
    clippy::excessive_precision,
    reason = "published AS241 coefficients, transcribed verbatim"
)]
const AS241_D: [f64; 8] = [
    0.0,
    2.053_191_626_637_758_821_87,
    1.676_384_830_183_803_849_4,
    0.689_767_334_985_100_004_55,
    0.148_103_976_427_480_074_59,
    0.015_198_666_563_616_457_196_6,
    5.475_938_084_995_344_946e-4,
    1.050_750_071_644_416_843_24e-9,
];
#[expect(
    clippy::excessive_precision,
    reason = "published AS241 coefficients, transcribed verbatim"
)]
const AS241_E: [f64; 8] = [
    6.657_904_643_501_103_777_2,
    5.463_784_911_164_114_369_9,
    1.784_826_539_917_291_335_8,
    0.296_560_571_828_504_891_23,
    0.026_532_189_526_576_123_093,
    0.001_242_660_947_388_078_438_6,
    2.711_555_568_743_487_578_15e-5,
    2.010_334_399_292_288_132_65e-7,
];
#[expect(
    clippy::excessive_precision,
    reason = "published AS241 coefficients, transcribed verbatim"
)]
const AS241_F: [f64; 8] = [
    0.0,
    0.599_832_206_555_887_937_69,
    0.136_929_880_922_735_805_31,
    0.014_875_361_290_850_614_852_5,
    7.868_691_311_456_132_591e-4,
    1.846_318_317_510_054_681_8e-5,
    1.421_511_758_316_445_888_7e-7,
    2.044_263_103_389_939_785_64e-15,
];

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
