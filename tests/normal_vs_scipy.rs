// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

//! R5: measured agreement of the normal CDF and PPF with SciPy (Q4, Q5).
//!
//! Prints the agreement statistics for each candidate so the decision in
//! `spec/queries.md` is grounded in a measurement, then asserts the bounds
//! recorded there. Run with `--nocapture` to see the report.

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

/// Candidate B for the CDF: the one-line erfc form.
fn cdf_erfc_only(z: f64) -> f64 {
    0.5 * libm::erfc(-z * std::f64::consts::FRAC_1_SQRT_2)
}

/// Tails where `|z|` is this large are bounded by relative error, not ulps:
/// there the CDF is below 1e-300 and subnormal.
const DEEP_TAIL: f64 = 37.0;

#[test]
fn normal_distribution_agreement_with_scipy() {
    let golden: Golden<Grid> = read_json("expected/normal-grid.json");
    eprintln!("reference environment: {}", golden.environment);

    let mut cdf_a = Agreement::default();
    let mut cdf_b = Agreement::default();
    for case in golden.cases.cdf.iter().filter(|c| c.z.abs() < DEEP_TAIL) {
        cdf_a.record(normal::cdf(case.z), case.cdf);
        cdf_b.record(cdf_erfc_only(case.z), case.cdf);
    }
    eprintln!(
        "cdf  A (Cephes ndtr structure, libm erf/erfc): {}",
        cdf_a.summary()
    );
    eprintln!(
        "cdf  B (0.5 * erfc(-z / sqrt 2)):              {}",
        cdf_b.summary()
    );

    let mut cdf_clinical = Agreement::default();
    for case in golden
        .cases
        .cdf
        .iter()
        .filter(|c| c.z.abs() <= CLINICAL_SDS)
    {
        cdf_clinical.record(normal::cdf(case.z), case.cdf);
    }
    eprintln!(
        "cdf  A, |z| <= 8 only:                         {}",
        cdf_clinical.summary()
    );

    let mut ppf = Agreement::default();
    let mut ppf_clinical = Agreement::default();
    let clinical_p = normal::cdf(-CLINICAL_SDS)..=normal::cdf(CLINICAL_SDS);
    for case in &golden.cases.ppf {
        ppf.record(normal::ppf(case.p), case.ppf);
        if clinical_p.contains(&case.p) {
            ppf_clinical.record(normal::ppf(case.p), case.ppf);
        }
    }
    eprintln!(
        "ppf  AS241 PPND16:                             {}",
        ppf.summary()
    );
    eprintln!(
        "ppf  AS241, p within +/-8 SD only:             {}",
        ppf_clinical.summary()
    );

    assert!(cdf_a.max_rel < CDF_MAX_REL, "cdf: {}", cdf_a.summary());
    assert!(ppf.max_rel < PPF_MAX_REL, "ppf: {}", ppf.summary());
}

/// The +/-8 SD range the engine accepts for height, weight, and OFC.
const CLINICAL_SDS: f64 = 8.0;

/// Bounds recorded against Q4 from measurement (2026-11-17, SciPy 1.18.1,
/// exact float parsing): CDF max relative difference 5.7e-14 over |z| < 37
/// (2.5e-15 within +/-8 SD), PPF 8.4e-16. The bounds sit just above those
/// values; a regression past them fails. Neither is a licence to widen:
/// investigate any increase.
const CDF_MAX_REL: f64 = 1e-13;
const PPF_MAX_REL: f64 = 2e-15;
