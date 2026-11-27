// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

#![cfg(all(feature = "server", feature = "embedded-data"))]

//! The request body limit (`spec/standard.md` section 7.4). The RCPCH server
//! reads a body of any size, which was checked by hand on 2026-11-27 with
//! bodies of 1, 2.5, and 10 MB: there is no golden, since one would hold
//! the body. opengrowth reads at most `MAX_REQUEST_BODY_BYTES`.

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use opengrowth::server::{AppState, MAX_REQUEST_BODY_BYTES, router};
use opengrowth::store::ReferenceStore;
use tower::ServiceExt;

/// Where opengrowth deliberately answers differently from the RCPCH server.
/// Each is named on docs/differences.md (`s/check-differences`).
const KNOWN_DIFFERENCES: &[&str] = &["a request body over 2 MiB"];

/// A valid calculation request, padded with spaces to exactly `size` bytes.
fn padded(size: usize) -> Vec<u8> {
    let request = r#"{"birth_date": "2020-04-12", "observation_date": "2028-06-12", "observation_value": 115, "sex": "female", "gestation_weeks": 40, "gestation_days": 0, "measurement_method": "height"}"#;
    let mut body = request.as_bytes()[..request.len() - 1].to_vec();
    body.resize(size - 1, b' ');
    body.push(b'}');
    body
}

async fn send(body: Vec<u8>) -> (StatusCode, Option<String>, String) {
    let app = router(Arc::new(AppState::new(
        ReferenceStore::standard(None).expect("tables load"),
        None,
    )));
    let request = Request::builder()
        .method("POST")
        .uri("/uk-who/calculation")
        .header("host", "testserver")
        .header("origin", "https://app.example.org")
        .header("content-type", "application/json")
        .body(Body::from(body))
        .expect("request");
    let response = app.oneshot(request).await.expect("response");
    let status = response.status();
    let allow_origin = response
        .headers()
        .get("access-control-allow-origin")
        .map(|value| value.to_str().expect("ASCII").to_owned());
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("body")
        .to_bytes();
    (
        status,
        allow_origin,
        String::from_utf8_lossy(&bytes).into_owned(),
    )
}

#[tokio::test]
async fn a_body_at_the_limit_is_read() {
    let (status, _, body) = send(padded(MAX_REQUEST_BODY_BYTES)).await;
    assert_eq!(status, StatusCode::OK, "{body}");
}

#[tokio::test]
async fn a_body_over_the_limit_is_refused() {
    assert_eq!(KNOWN_DIFFERENCES, ["a request body over 2 MiB"]);
    assert_eq!(MAX_REQUEST_BODY_BYTES, 2 * 1024 * 1024);
    let (status, allow_origin, body) = send(padded(MAX_REQUEST_BODY_BYTES + 1)).await;
    assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
    assert_eq!(
        body,
        "Failed to buffer the request body: length limit exceeded"
    );
    // A browser can read the refusal: CORS still applies.
    assert_eq!(allow_origin.as_deref(), Some("*"));
}
