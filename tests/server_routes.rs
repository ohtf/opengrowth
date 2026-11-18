// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

#![cfg(feature = "server")]

//! Server behaviour that needs no reference tables, so it runs in CI.
//! Expected bodies are copied from `conformance/expected/http-uk-who.json`,
//! which `s/conformance` checks in full.

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use opengrowth::server::{AppState, router};
use opengrowth::store::ReferenceStore;
use tower::ServiceExt;

async fn send(
    key: Option<&str>,
    method: &str,
    path: &str,
    body: &str,
    auth: Option<&str>,
) -> (StatusCode, String, String) {
    let app = router(Arc::new(AppState::new(
        ReferenceStore::empty(),
        key.map(str::to_owned),
    )));
    let mut request = Request::builder()
        .method(method)
        .uri(path)
        .header("host", "testserver")
        .header("content-type", "application/json");
    if let Some(auth) = auth {
        request = request.header("authorization", auth);
    }
    let response = app
        .oneshot(request.body(Body::from(body.to_owned())).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let location = response
        .headers()
        .get("location")
        .map(|v| v.to_str().unwrap().to_owned())
        .unwrap_or_default();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (status, String::from_utf8(bytes.to_vec()).unwrap(), location)
}

const VALID: &str = r#"{"birth_date": "2020-04-12", "observation_date": "2028-06-12", "observation_value": 115, "sex": "female", "gestation_weeks": 40, "gestation_days": 0, "measurement_method": "height"}"#;

#[tokio::test]
async fn validation_errors_are_byte_identical_to_upstream() {
    let (status, body, _) = send(None, "POST", "/uk-who/calculation", "{}", None).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        body,
        r#"{"detail":[{"type":"missing","loc":["body","observation_date"],"msg":"Field required","input":{}},{"type":"missing","loc":["body","observation_value"],"msg":"Field required","input":{}},{"type":"missing","loc":["body","measurement_method"],"msg":"Field required","input":{}},{"type":"missing","loc":["body","birth_date"],"msg":"Field required","input":{}},{"type":"missing","loc":["body","sex"],"msg":"Field required","input":{}}]}"#
    );

    let two = VALID
        .replace(r#""sex": "female""#, r#""sex": "x""#)
        .replace(r#""gestation_weeks": 40"#, r#""gestation_weeks": 50"#);
    let (_, body, _) = send(None, "POST", "/uk-who/calculation", &two, None).await;
    assert_eq!(
        body,
        r#"{"detail":[{"type":"less_than_equal","loc":["body","gestation_weeks"],"msg":"Input should be less than or equal to 44","input":50,"ctx":{"le":44}},{"type":"literal_error","loc":["body","sex"],"msg":"Input should be 'male' or 'female'","input":"x","ctx":{"expected":"'male' or 'female'"}}]}"#
    );

    let (_, body, _) = send(
        None,
        "POST",
        "/uk-who/calculation",
        r#"{"sex": "male",}"#,
        None,
    )
    .await;
    assert_eq!(
        body,
        r#"{"detail":[{"type":"json_invalid","loc":["body",14],"msg":"JSON decode error","input":{},"ctx":{"error":"Illegal trailing comma before end of object"}}]}"#
    );

    let (_, body, _) = send(None, "POST", "/uk-who/calculation", "", None).await;
    assert_eq!(
        body,
        r#"{"detail":[{"type":"missing","loc":["body"],"msg":"Field required","input":null}]}"#
    );
}

#[tokio::test]
async fn bulk_count_limits_are_route_errors() {
    let (status, body, _) = send(
        None,
        "POST",
        "/uk-who/bulk-calculation",
        r#"{"measurement_method": "height", "birth_date": "2020-04-12", "sex": "female", "observations": []}"#,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        body,
        r#"{"detail":[{"loc":["body"],"msg":"At least one observation is required for bulk calculation.","type":"value_error","input":"observations"}]}"#
    );
}

#[tokio::test]
async fn routing_matches_starlette() {
    let (status, body, _) = send(None, "POST", "/nope", VALID, None).await;
    assert_eq!(
        (status, body.as_str()),
        (StatusCode::NOT_FOUND, r#"{"detail":"Not Found"}"#)
    );
    let (status, body, _) = send(None, "GET", "/uk-who/calculation", "", None).await;
    assert_eq!(
        (status, body.as_str()),
        (
            StatusCode::METHOD_NOT_ALLOWED,
            r#"{"detail":"Method Not Allowed"}"#
        )
    );
    let (status, _, _) = send(None, "GET", "/uk-who/chart-coordinates", "", None).await;
    assert_eq!(status, StatusCode::METHOD_NOT_ALLOWED);
    let (status, body, location) = send(None, "POST", "/uk-who/calculation/", VALID, None).await;
    assert_eq!(status, StatusCode::TEMPORARY_REDIRECT);
    assert_eq!(
        (body.as_str(), location.as_str()),
        ("", "http://testserver/uk-who/calculation")
    );
}

#[tokio::test]
async fn authorisation_is_a_bearer_token() {
    let (status, body, _) = send(Some("k"), "POST", "/uk-who/calculation", VALID, None).await;
    assert_eq!(
        (status, body.as_str()),
        (StatusCode::FORBIDDEN, r#"{"detail":"Forbidden"}"#)
    );
    let (status, _, _) = send(
        Some("k"),
        "POST",
        "/uk-who/calculation",
        VALID,
        Some("Bearer k"),
    )
    .await;
    assert_ne!(status, StatusCode::FORBIDDEN);
    let (status, _, _) = send(Some("k"), "GET", "/", "", None).await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn missing_tables_are_reported_never_substituted() {
    let (status, body, _) = send(None, "POST", "/uk-who/calculation", VALID, None).await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert!(body.contains("uk90-child"), "{body}");
    let chart = r#"{"sex": "male", "measurement_method": "height"}"#;
    let (status, body, _) = send(None, "POST", "/uk-who/chart-coordinates", chart, None).await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert!(body.contains("uk90-preterm"), "{body}");
}

#[tokio::test]
async fn unported_operations_say_so() {
    let (status, _, _) = send(None, "POST", "/cdc/calculation", VALID, None).await;
    assert_eq!(status, StatusCode::NOT_IMPLEMENTED);
}
