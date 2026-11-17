// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-FileCopyrightText: Royal College of Paediatrics and Child Health
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The LMS method (Cole and Green, 1992): converting between a measurement
//! and its SDS given age-specific L (Box-Cox power), M (median), and S
//! (coefficient of variation) values.
//!
//! Expressions are transcribed from `rcpchgrowth.global_functions.z_score`
//! and `measurement_for_z` in the same operation order, so that results
//! agree with the Python engine to the last bit where the platform maths
//! library agrees. Do not simplify them algebraically.

/// The L, M, and S values for one age, plus the CDC extended-BMI sigma where
/// the reference has one.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Lms {
    /// Box-Cox power.
    pub l: f64,
    /// Median.
    pub m: f64,
    /// Coefficient of variation.
    pub s: f64,
    /// CDC extended-BMI sigma, present only in CDC BMI tables.
    pub sigma: Option<f64>,
}

/// SDS (z-score) for an observation, given L, M, and S.
///
/// `z = ((x / M)^L - 1) / (L S)`, or `ln(x / M) / S` when `L` is zero.
///
/// The caller must ensure `observation` is positive. Python raises or
/// returns a complex number for a non-positive observation; the engine
/// rejects it before reaching this function.
#[must_use]
pub fn sds_from_lms(observation: f64, lms: &Lms) -> f64 {
    let Lms { l, m, s, .. } = *lms;
    if l != 0.0 {
        ((observation / m).powf(l) - 1.0) / (l * s)
    } else {
        (observation / m).ln() / s
    }
}

/// Measurement for an SDS, given L, M, and S.
///
/// `x = M (1 + L S z)^(1/L)`, or `M e^(S z)` when `L` is zero.
///
/// Returns `None` where Python's `measurement_for_z` returns `None`: when
/// `1 + L S z` is negative, and when it is zero with a negative exponent
/// (which Python reports as a `ZeroDivisionError` and swallows).
#[must_use]
pub fn measurement_from_lms(z: f64, lms: &Lms) -> Option<f64> {
    let Lms { l, m, s, .. } = *lms;
    if l != 0.0 {
        let first_step = 1.0 + (l * s * z);
        let exponent = 1.0 / l;
        if first_step < 0.0 || (first_step == 0.0 && exponent < 0.0) {
            return None;
        }
        Some(first_step.powf(exponent) * m)
    } else {
        Some((s * z).exp() * m)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LMS: Lms = Lms {
        l: 0.3487,
        m: 3.3464,
        s: 0.14602,
        sigma: None,
    };

    #[test]
    fn median_has_zero_sds() {
        assert_eq!(sds_from_lms(LMS.m, &LMS), 0.0);
        assert_eq!(measurement_from_lms(0.0, &LMS), Some(LMS.m));
    }

    #[test]
    fn conversions_are_inverse() {
        for z in [-4.0, -2.0, -0.67, 0.5, 1.33, 3.0] {
            let x = measurement_from_lms(z, &LMS).unwrap();
            assert!((sds_from_lms(x, &LMS) - z).abs() < 1e-12, "z = {z}");
        }
    }

    #[test]
    fn zero_l_uses_the_log_form() {
        let lms = Lms { l: 0.0, ..LMS };
        let x = measurement_from_lms(1.0, &lms).unwrap();
        assert!((sds_from_lms(x, &lms) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn impossible_measurements_return_none() {
        // 1 + L S z < 0
        assert_eq!(measurement_from_lms(-30.0, &LMS), None);
        // 1 + L S z == 0 with a negative exponent
        let lms = Lms {
            l: -1.0,
            m: 1.0,
            s: 0.5,
            sigma: None,
        };
        assert_eq!(measurement_from_lms(2.0, &lms), None);
    }
}
