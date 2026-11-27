// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Replaying HTTP goldens captured from the FastAPI server, shared by the
//! HTTP conformance suites.

use std::collections::BTreeMap;

use axum::body::Body;
use axum::http::Request;
use http_body_util::BodyExt;
use tower::ServiceExt;

use super::{mask_provenance, sha256_hex};

/// One captured request and response.
#[derive(Debug, serde::Deserialize)]
pub struct Case {
    pub name: String,
    pub method: String,
    pub path: String,
    pub request_headers: BTreeMap<String, String>,
    pub request_body: String,
    /// The body, in hex, where it is not UTF-8; `request_body` is then empty.
    #[serde(default)]
    pub request_body_hex: Option<String>,
    pub status: u16,
    pub headers: BTreeMap<String, String>,
    pub body_sha256: String,
    pub masked_sha256: String,
    pub body: Option<String>,
}

/// What opengrowth returned for a case.
#[derive(Debug)]
pub struct Outcome {
    pub status: u16,
    pub headers: BTreeMap<String, String>,
    pub body: String,
}

/// Sends a case's request to `app`, as JSON.
pub async fn replay(app: axum::Router, case: &Case) -> Outcome {
    send(app, case, Some("application/json")).await
}

/// Sends a case's request with exactly the headers it records, and no
/// `Content-Type` unless it records one.
pub async fn replay_exact(app: axum::Router, case: &Case) -> Outcome {
    send(app, case, None).await
}

async fn send(app: axum::Router, case: &Case, content_type: Option<&str>) -> Outcome {
    let mut request = Request::builder()
        .method(case.method.as_str())
        .uri(case.path.as_str())
        .header("host", "testserver");
    if let Some(content_type) = content_type {
        request = request.header("content-type", content_type);
    }
    for (name, value) in &case.request_headers {
        request = request.header(name.as_str(), value.as_str());
    }
    let body = match &case.request_body_hex {
        Some(hex) => (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).expect("a hex body"))
            .collect(),
        None => case.request_body.clone().into_bytes(),
    };
    let request = request.body(Body::from(body)).expect("request");
    let response = app.oneshot(request).await.expect("infallible");
    let status = response.status().as_u16();
    let headers = ["content-type", "allow", "location", "x-git-revision"]
        .into_iter()
        .filter_map(|h| {
            response
                .headers()
                .get(h)
                .map(|v| (h.to_owned(), v.to_str().unwrap_or("").to_owned()))
        })
        .collect();
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("body")
        .to_bytes();
    Outcome {
        status,
        headers,
        body: String::from_utf8(bytes.to_vec()).expect("UTF-8 body"),
    }
}

/// Every way `outcome` differs from `case`, ignoring provenance; empty if it
/// matches byte for byte. Also checks that any provenance names the route's
/// reference and opengrowth.
pub fn problems(case: &Case, outcome: &Outcome) -> Vec<String> {
    let mut problems = Vec::new();
    if outcome.status != case.status {
        problems.push(format!("status {} != {}", outcome.status, case.status));
    }
    if outcome.headers != case.headers {
        problems.push(format!(
            "headers {:?} != {:?}",
            outcome.headers, case.headers
        ));
    }
    let masked = mask_provenance(&outcome.body);
    if sha256_hex(masked.as_bytes()) != case.masked_sha256 {
        match &case.body {
            Some(expected) => problems.push(first_difference(&mask_provenance(expected), &masked)),
            None => problems.push(format!(
                "masked body differs (digest only); ours starts {}",
                &masked[..masked.len().min(300)]
            )),
        }
    }
    if outcome.status == 200 && outcome.body.contains("\"provenance\"") {
        let reference = expected_reference(case);
        let ok = outcome
            .body
            .contains(&format!("\"growth_reference\":\"{reference}\""))
            && outcome
                .body
                .contains("\"calculation_engine\":{\"name\":\"opengrowth\"");
        if !ok {
            problems.push(format!(
                "provenance does not identify {reference} and opengrowth"
            ));
        }
    }
    problems
}

/// The reference a successful response must name in its provenance: the
/// route's own, or for `/utilities` the request's `reference` (default `uk-who`).
fn expected_reference(case: &Case) -> String {
    let first = case
        .path
        .trim_start_matches('/')
        .split('/')
        .next()
        .unwrap_or("");
    if first == "turner" {
        return "turners-syndrome".to_owned();
    }
    if first != "utilities" {
        return first.to_owned();
    }
    serde_json::from_str::<serde_json::Value>(&case.request_body)
        .ok()
        .and_then(|body| {
            body.get("reference")
                .and_then(|r| r.as_str())
                .map(str::to_owned)
        })
        .unwrap_or_else(|| "uk-who".to_owned())
}

/// Whether the whole body, provenance included, is identical.
pub fn identical_including_provenance(case: &Case, outcome: &Outcome) -> bool {
    sha256_hex(outcome.body.as_bytes()) == case.body_sha256
}

/// The first point where two strings differ, with context.
pub fn first_difference(expected: &str, actual: &str) -> String {
    let at = expected
        .char_indices()
        .zip(actual.chars())
        .find(|((_, a), b)| a != b)
        .map_or(expected.len().min(actual.len()), |((i, _), _)| i);
    let from = at.saturating_sub(80);
    let window = |s: &str| s.get(from..(at + 80).min(s.len())).unwrap_or("").to_owned();
    format!(
        "at byte {at}:\n    expected …{}…\n    actual   …{}…",
        window(expected),
        window(actual)
    )
}

/// Fails with the first few differences, if there are any.
pub fn assert_no_failures(failures: &[String], total: usize) {
    assert!(
        failures.is_empty(),
        "{} of {total} cases differ:\n{}",
        failures.len(),
        failures
            .iter()
            .take(25)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// A reference's calculation golden: curated cases, then the R-vector children.
#[derive(Debug, serde::Deserialize)]
pub struct ReferenceCases {
    pub curated: Vec<Case>,
    pub r_vectors: Vec<Case>,
}

/// Replays a reference's calculation and bulk-calculation golden against the
/// embedded tables (or `$OPENGROWTH_REFERENCE_DATA_DIR` layered over them)
/// and fails on any difference outside provenance. Each entry of `known` is
/// a Category 3 fix (`spec/standard.md` section 6.3) that must still differ
/// in exactly the stated way, upstream 500 and opengrowth 422, so the list
/// cannot go stale or hide a regression.
pub async fn check_reference_golden(golden_file: &str, known: &[(&str, &str)]) {
    use std::sync::Arc;

    use opengrowth::server::{AppState, router};
    use opengrowth::store::ReferenceStore;

    let dir = std::env::var_os(super::DATA_DIR_ENV).map(std::path::PathBuf::from);
    let store = || ReferenceStore::standard(dir.as_deref()).expect("tables load");
    let open = router(Arc::new(AppState::new(store(), None)));
    let locked = router(Arc::new(AppState::new(
        store(),
        Some("test-key".to_owned()),
    )));

    let golden: super::Golden<ReferenceCases> = super::read_json(golden_file);
    eprintln!("reference server environment: {}", golden.environment);
    let mut failures = Vec::new();
    let mut identical = 0;
    let mut masked_identical = 0;
    let all: Vec<&Case> = golden
        .cases
        .curated
        .iter()
        .chain(&golden.cases.r_vectors)
        .collect();
    for case in &all {
        let app = if case.name.starts_with("auth ") {
            locked.clone()
        } else {
            open.clone()
        };
        let outcome = replay(app, case).await;
        if known.iter().any(|(name, _)| *name == case.name) {
            let as_expected = case.status == 500
                && outcome.status == 422
                && outcome.body.contains("\"value_error\"");
            if !as_expected {
                failures.push(format!(
                    "{}: listed as a known difference but behaved differently",
                    case.name
                ));
            }
            continue;
        }
        if identical_including_provenance(case, &outcome) {
            identical += 1;
        }
        let found = problems(case, &outcome);
        if found.is_empty() {
            masked_identical += 1;
        } else {
            failures.push(format!("{}: {}", case.name, found.join("; ")));
        }
    }
    eprintln!(
        "{} cases: {masked_identical} byte-identical outside provenance, {identical} identical including it, {} known differences",
        all.len(),
        known.len()
    );
    assert_no_failures(&failures, all.len());
}

/// One chart case, with the digest the unmodified server returns for a
/// named format (from its stale stored files).
#[derive(Debug, serde::Deserialize)]
pub struct ChartCase {
    #[serde(flatten)]
    pub case: Case,
    pub stale_masked_sha256: Option<String>,
}

/// The chart golden's one known difference: upstream serves stored files
/// that differ from its own engine.
pub const STALE_CHART_FILES: (&str, &str) = (
    "every named-format chart",
    "upstream serves chart files last regenerated on 2025-01-11, which differ from its own engine; \
     opengrowth calculates them with rcpchgrowth 4.6.5",
);

/// Replays a reference's chart golden and fails on any difference outside
/// provenance. With `stale_files`, also checks that that known difference
/// still holds: every named-format response from the unmodified server
/// differs from 4.6.5's. Without it, none may: the stored files are current.
pub async fn check_chart_golden(golden_file: &str, known: &[(&str, &str)], stale_files: bool) {
    use std::sync::Arc;

    use opengrowth::server::{AppState, router};
    use opengrowth::store::ReferenceStore;

    let dir = std::env::var_os(super::DATA_DIR_ENV).map(std::path::PathBuf::from);
    let app = router(Arc::new(AppState::new(
        ReferenceStore::standard(dir.as_deref()).expect("tables load"),
        None,
    )));
    let golden: super::Golden<Vec<ChartCase>> = super::read_json(golden_file);
    let mut failures = Vec::new();
    let mut stale_differs = 0;
    let mut named = 0;
    let mut seen = 0;
    for chart_case in &golden.cases {
        let case = &chart_case.case;
        let outcome = replay(app.clone(), case).await;
        if let Some((_, reason)) = known.iter().find(|(name, _)| *name == case.name) {
            seen += 1;
            if !(case.status == 500 && outcome.status == 422) {
                failures.push(format!(
                    "{}: listed as a known difference ({reason}) but returned {}",
                    case.name, outcome.status
                ));
            }
            continue;
        }
        let found = problems(case, &outcome);
        if !found.is_empty() {
            failures.push(format!("{}: {}", case.name, found.join("; ")));
        }
        // Only responses the route serves from its files can be stale; the
        // others are errors, identical either way.
        if let Some(stale) = chart_case
            .stale_masked_sha256
            .as_ref()
            .filter(|_| case.status == 200)
        {
            named += 1;
            if *stale != case.masked_sha256 {
                stale_differs += 1;
            }
        }
    }
    eprintln!(
        "{} cases; {} differ. Upstream's stored chart files differ from rcpchgrowth 4.6.5 for {stale_differs} of {named} named formats.",
        golden.cases.len(),
        failures.len()
    );
    assert_eq!(seen, known.len(), "a known difference has no case");
    let (name, reason) = STALE_CHART_FILES;
    if stale_files && stale_differs != named {
        failures.push(format!(
            "{name} is listed as a known difference ({reason}), but only {stale_differs} of {named} differ"
        ));
    }
    if !stale_files && stale_differs != 0 {
        failures.push(format!(
            "{stale_differs} of {named} stored chart files differ from 4.6.5, which is not a listed difference"
        ));
    }
    assert_no_failures(&failures, golden.cases.len());
}

/// The draw `conformance/generators/_http_fictional_child.py` fixes.
pub const FIXED_DRAW: f64 = 0.8125;

/// Replays a reference's fictional-child golden with the same fixed random
/// draw. Each entry of `known` (`spec/standard.md` section 6.3) must still be
/// upstream's 500 and opengrowth's route 422 with message `route_error`.
pub async fn check_fictional_child_golden(
    golden_file: &str,
    known: &[(&str, &str)],
    route_error: &str,
) {
    use std::sync::Arc;

    use opengrowth::server::{AppState, router};
    use opengrowth::store::ReferenceStore;

    let dir = std::env::var_os(super::DATA_DIR_ENV).map(std::path::PathBuf::from);
    let store = ReferenceStore::standard(dir.as_deref()).expect("tables load");
    let app = router(Arc::new(
        AppState::new(store, None).with_fixed_noise(FIXED_DRAW),
    ));
    let golden: super::Golden<Vec<Case>> = super::read_json(golden_file);
    assert!(
        golden.environment.to_string().contains("rcpchgrowth"),
        "unexpected golden environment"
    );
    let mut failures = Vec::new();
    let mut seen = 0;
    for case in &golden.cases {
        let outcome = replay(app.clone(), case).await;
        if let Some((_, reason)) = known.iter().find(|(name, _)| *name == case.name) {
            seen += 1;
            if !(case.status == 500 && outcome.status == 422 && outcome.body == route_error) {
                failures.push(format!(
                    "{}: listed as a known difference ({reason}) but behaved differently: {} {}",
                    case.name, outcome.status, outcome.body
                ));
            }
            continue;
        }
        let found = problems(case, &outcome);
        if !found.is_empty() {
            failures.push(format!("{}: {}", case.name, found.join("; ")));
        }
    }
    eprintln!(
        "{} cases: {} byte-identical outside provenance, {seen} known differences",
        golden.cases.len(),
        golden.cases.len() - seen - failures.len()
    );
    assert_eq!(seen, known.len(), "a known difference has no case");
    assert_no_failures(&failures, golden.cases.len());
}
