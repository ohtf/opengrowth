// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

#![cfg(all(feature = "server", feature = "embedded-data"))]

//! Profile B for UK-WHO: replays every request captured from the FastAPI
//! server and compares status, recorded headers, and the body byte for byte
//! outside `provenance` (`spec/standard.md` section 4.2).
//!
//! Uses the embedded tables, or the files in `$OPENGROWTH_REFERENCE_DATA_DIR`
//! layered over them if it is set.

mod common;

use std::collections::BTreeMap;
use std::sync::Arc;

use axum::body::Body;
use axum::http::Request;
use common::{DATA_DIR_ENV, Golden, mask_provenance, read_json, sha256_hex};
use http_body_util::BodyExt;
use opengrowth::server::{AppState, router};
use opengrowth::store::ReferenceStore;
use tower::ServiceExt;

#[derive(serde::Deserialize)]
struct Case {
    name: String,
    method: String,
    path: String,
    request_headers: BTreeMap<String, String>,
    request_body: String,
    status: u16,
    headers: BTreeMap<String, String>,
    body_sha256: String,
    masked_sha256: String,
    body: Option<String>,
}

#[derive(serde::Deserialize)]
struct Cases {
    curated: Vec<Case>,
    r_vectors: Vec<Case>,
}

struct Outcome {
    status: u16,
    headers: BTreeMap<String, String>,
    body: String,
}

async fn replay(app: axum::Router, case: &Case) -> Outcome {
    let mut request = Request::builder()
        .method(case.method.as_str())
        .uri(case.path.as_str())
        .header("host", "testserver")
        .header("content-type", "application/json");
    for (name, value) in &case.request_headers {
        request = request.header(name.as_str(), value.as_str());
    }
    let request = request
        .body(Body::from(case.request_body.clone()))
        .expect("request");
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

/// Cases where opengrowth deliberately differs from upstream, each a
/// Category 3 fix listed in `spec/standard.md` section 6.3. Each must still
/// differ in exactly the stated way: upstream 500, opengrowth 422. A case
/// that starts to match, or differs differently, fails the test, so this
/// list cannot go stale or hide a regression.
const KNOWN_DIFFERENCES: &[(&str, &str)] = &[
    (
        "validation birth date not string",
        "upstream strptime raises TypeError, which escapes as a 500",
    ),
    (
        "validation birth date null",
        "upstream strptime raises TypeError, which escapes as a 500",
    ),
];

/// The first point where two strings differ, with context.
fn first_difference(expected: &str, actual: &str) -> String {
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

#[tokio::test]
async fn uk_who_http_matches_fastapi_byte_for_byte() {
    let dir = std::env::var_os(DATA_DIR_ENV).map(std::path::PathBuf::from);
    let store = || ReferenceStore::standard(dir.as_deref()).expect("tables load");
    let open = router(Arc::new(AppState {
        store: store(),
        authorization_key: None,
    }));
    let locked = router(Arc::new(AppState {
        store: store(),
        authorization_key: Some("test-key".to_owned()),
    }));

    let golden: Golden<Cases> = read_json("expected/http-uk-who.json");
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
        if KNOWN_DIFFERENCES.iter().any(|(name, _)| *name == case.name) {
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
        if sha256_hex(outcome.body.as_bytes()) == case.body_sha256 {
            identical += 1;
        }
        if sha256_hex(masked.as_bytes()) == case.masked_sha256 {
            masked_identical += 1;
        } else {
            match &case.body {
                Some(expected) => {
                    problems.push(first_difference(&mask_provenance(expected), &masked))
                }
                None => problems.push(format!(
                    "masked body differs (digest only); ours starts {}",
                    &masked[..masked.len().min(300)]
                )),
            }
        }
        if outcome.status == 200 && outcome.body.contains("\"provenance\"") {
            let ok = outcome.body.contains("\"growth_reference\":\"uk-who\"")
                && outcome
                    .body
                    .contains("\"calculation_engine\":{\"name\":\"opengrowth\"");
            if !ok {
                problems.push("provenance does not identify uk-who and opengrowth".to_owned());
            }
        }
        if !problems.is_empty() {
            failures.push(format!("{}: {}", case.name, problems.join("; ")));
        }
    }
    eprintln!(
        "{} cases: {masked_identical} byte-identical outside provenance, {identical} identical including it, {} known differences",
        all.len(),
        KNOWN_DIFFERENCES.len()
    );
    assert!(
        failures.is_empty(),
        "{} of {} cases differ:\n{}",
        failures.len(),
        all.len(),
        failures
            .iter()
            .take(25)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}
