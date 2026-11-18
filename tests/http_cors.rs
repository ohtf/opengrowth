// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

#![cfg(all(feature = "server", feature = "embedded-data"))]

//! CORS against the pinned server (`spec/queries.md` Q8). The golden records
//! each request twice: from the server as pinned, and from the same server
//! with its CORS middleware set to `allow_credentials=False` and placed in
//! front of authorisation ("resolved"). opengrowth must match the resolved
//! server exactly, and the pinned server must differ from it only by the
//! three rules in [`pinned_from_resolved`], so the difference stays measured.

mod common;

use std::collections::BTreeMap;
use std::sync::Arc;

use axum::body::Body;
use axum::http::Request;
use common::{Golden, read_json};
use http_body_util::BodyExt as _;
use opengrowth::server::{AppState, router};
use opengrowth::store::ReferenceStore;
use tower::ServiceExt as _;

/// The headers the generator records.
const RECORDED: [&str; 9] = [
    "access-control-allow-origin",
    "access-control-allow-credentials",
    "access-control-allow-methods",
    "access-control-allow-headers",
    "access-control-max-age",
    "access-control-expose-headers",
    "vary",
    "content-type",
    "allow",
];

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
struct Observation {
    status: u16,
    headers: BTreeMap<String, String>,
    /// Short bodies only; long ones are compared by the other HTTP suites.
    body: Option<String>,
    body_sha256: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct CorsCase {
    name: String,
    key: Option<String>,
    method: String,
    path: String,
    request_headers: BTreeMap<String, String>,
    request_body: String,
    pinned: Observation,
    resolved: Observation,
}

async fn observe(app: axum::Router, case: &CorsCase) -> (u16, BTreeMap<String, String>, String) {
    let mut request = Request::builder()
        .method(case.method.as_str())
        .uri(case.path.as_str())
        .header("host", "testserver");
    if !case.request_body.is_empty() {
        request = request.header("content-type", "application/json");
    }
    for (name, value) in &case.request_headers {
        request = request.header(name.as_str(), value.as_str());
    }
    let request = request
        .body(Body::from(case.request_body.clone()))
        .expect("request");
    let response = app.oneshot(request).await.expect("infallible");
    let status = response.status().as_u16();
    let headers = RECORDED
        .iter()
        .filter_map(|h| {
            let value = response.headers().get(*h)?.to_str().ok()?;
            Some(((*h).to_owned(), value.to_owned()))
        })
        .collect();
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("body")
        .to_bytes();
    (
        status,
        headers,
        String::from_utf8_lossy(&bytes).into_owned(),
    )
}

#[tokio::test]
async fn cors_matches_upstream_with_its_contradiction_resolved() {
    let golden: Golden<Vec<CorsCase>> = read_json("expected/http-cors.json");
    let store = || ReferenceStore::standard(None).expect("tables load");
    let mut failures = Vec::new();
    for case in &golden.cases {
        let app = router(Arc::new(AppState::new(store(), case.key.clone())));
        let (status, headers, body) = observe(app, case).await;
        let expected = &case.resolved;
        if status != expected.status || headers != expected.headers {
            failures.push(format!(
                "{} (key {:?}): {status} {headers:?}, expected {} {:?}",
                case.name, case.key, expected.status, expected.headers
            ));
        } else if expected.body.as_ref().is_some_and(|b| *b != body) {
            failures.push(format!(
                "{} (key {:?}): body {body:?}, expected {:?}",
                case.name, case.key, expected.body
            ));
        }
    }
    eprintln!(
        "{} cases; {} differ from the resolved server",
        golden.cases.len(),
        failures.len()
    );
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// What the pinned server sends, derived from the resolved server's response
/// by the three differences Q8 records:
///
/// 1. It allows credentials, so every response with CORS headers also says
///    `access-control-allow-credentials: true`.
/// 2. Because of that, its preflights echo the caller's origin, with
///    `vary: Origin`, instead of saying `*`.
/// 3. Its authorisation runs before CORS, so with a key set, a request
///    without it to any path but `/`, preflights included, gets a bare 403.
fn pinned_from_resolved(case: &CorsCase) -> Observation {
    let header = |name: &str| case.request_headers.get(name);
    if let Some(key) = &case.key
        && case.path != "/"
        && header("authorization") != Some(&format!("Bearer {key}"))
    {
        return Observation {
            status: 403,
            headers: BTreeMap::from([("content-type".to_owned(), "application/json".to_owned())]),
            body: Some(r#"{"detail":"Forbidden"}"#.to_owned()),
            body_sha256: None,
        };
    }
    let mut pinned = case.resolved.clone();
    let preflight = case.method == "OPTIONS"
        && header("origin").is_some()
        && header("access-control-request-method").is_some();
    if preflight {
        let origin = header("origin").expect("checked").clone();
        pinned
            .headers
            .insert("access-control-allow-origin".to_owned(), origin);
        pinned
            .headers
            .insert("vary".to_owned(), "Origin".to_owned());
    }
    if pinned.headers.contains_key("access-control-allow-origin") {
        pinned.headers.insert(
            "access-control-allow-credentials".to_owned(),
            "true".to_owned(),
        );
    }
    pinned
}

#[test]
fn the_pinned_server_differs_only_as_q8_records() {
    let golden: Golden<Vec<CorsCase>> = read_json("expected/http-cors.json");
    let mut differing = 0;
    for case in &golden.cases {
        assert_eq!(
            pinned_from_resolved(case),
            case.pinned,
            "{} (key {:?})",
            case.name,
            case.key
        );
        if case.pinned != case.resolved {
            differing += 1;
        }
    }
    eprintln!(
        "{} cases; the pinned server differs in {differing}, each by the recorded rules",
        golden.cases.len()
    );
}
