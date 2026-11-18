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

/// Sends a case's request to `app`.
pub async fn replay(app: axum::Router, case: &Case) -> Outcome {
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

/// Every way `outcome` differs from `case`, ignoring provenance; empty if it
/// matches byte for byte. Also checks that any provenance names uk-who and opengrowth.
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
        let ok = outcome.body.contains("\"growth_reference\":\"uk-who\"")
            && outcome
                .body
                .contains("\"calculation_engine\":{\"name\":\"opengrowth\"");
        if !ok {
            problems.push("provenance does not identify uk-who and opengrowth".to_owned());
        }
    }
    problems
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
