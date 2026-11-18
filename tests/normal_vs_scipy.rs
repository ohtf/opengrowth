// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

//! R5 and R32: agreement of the normal CDF and PPF with SciPy (Q4, Q5).
//!
//! The engine's `cdf` and `ppf` are translations of SciPy's own `ndtr` and
//! `ndtri`, so on Linux with glibc, which supplies the `exp` and `log` that
//! both use, they must agree with SciPy to the bit at every grid point,
//! including the deep tails. Elsewhere the platform maths library may
//! differ in the last bit, so the test asserts the relative bounds instead
//! (`R34`). Run with `--nocapture` to see the report.

mod common;

use common::{Agreement, Golden, read_json};
use opengrowth::normal;

#[derive(serde::Deserialize)]
struct Grid {
    cdf: Vec<CdfCase>,
    ppf: Vec<PpfCase>,
}

#[derive(serde::Deserialize)]
struct CdfCase {
    z: f64,
    cdf: f64,
}

#[derive(serde::Deserialize)]
struct PpfCase {
    p: f64,
    ppf: f64,
}

/// Whether this platform's maths library is the one SciPy was measured on.
const SAME_MATHS_LIBRARY_AS_SCIPY: bool = cfg!(all(target_os = "linux", target_env = "gnu"));

/// Bounds for other platforms, from the Cephes accuracy figures and the
/// measurement before `R32`: they hold for a correct implementation on any
/// correctly working maths library. Neither is a licence to widen.
const CDF_MAX_REL: f64 = 1e-13;
const PPF_MAX_REL: f64 = 2e-15;

/// Beyond this `|z|` the CDF is below 1e-300 and subnormal, so it has too
/// few significant bits for a relative bound.
const DEEP_TAIL: f64 = 37.0;

#[test]
fn normal_distribution_agreement_with_scipy() {
    let golden: Golden<Grid> = read_json("expected/normal-grid.json");
    eprintln!("reference environment: {}", golden.environment);

    let mut cdf = Agreement::default();
    let mut cdf_not_subnormal = Agreement::default();
    for case in &golden.cases.cdf {
        cdf.record(normal::cdf(case.z), case.cdf);
        if case.z.abs() < DEEP_TAIL {
            cdf_not_subnormal.record(normal::cdf(case.z), case.cdf);
        }
    }
    let mut ppf = Agreement::default();
    for case in &golden.cases.ppf {
        ppf.record(normal::ppf(case.p), case.ppf);
    }
    eprintln!("cdf (SciPy ndtr, translated): {}", cdf.summary());
    eprintln!("ppf (SciPy ndtri, translated): {}", ppf.summary());

    if SAME_MATHS_LIBRARY_AS_SCIPY {
        assert_eq!(cdf.bit_exact, cdf.compared, "cdf: {}", cdf.summary());
        assert_eq!(ppf.bit_exact, ppf.compared, "ppf: {}", ppf.summary());
    } else {
        assert!(
            cdf_not_subnormal.max_rel < CDF_MAX_REL,
            "cdf: {}",
            cdf_not_subnormal.summary()
        );
        assert!(ppf.max_rel < PPF_MAX_REL, "ppf: {}", ppf.summary());
    }
}
