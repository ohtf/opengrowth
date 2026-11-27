// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-FileCopyrightText: Royal College of Paediatrics and Child Health
// SPDX-License-Identifier: AGPL-3.0-or-later

#![cfg(feature = "server")]

//! The OpenAPI document (`R49`): the snapshot the documentation publishes is
//! current, every path it documents is a route the server answers, and it
//! differs from the pinned server's document only where listed here. Also
//! upstream's provenance case for the document (`R22`), ported from the
//! server's `tests/test_provenance.py`.

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use opengrowth::server::openapi::{self, default_servers};
use opengrowth::server::{AppState, router};
use opengrowth::store::ReferenceStore;
use serde_json::{Value, json};
use tower::ServiceExt;

fn ours() -> Value {
    openapi::document(&default_servers())
}

/// A path item without the words opengrowth writes for itself.
fn without_words(item: &Value) -> Value {
    let mut item = item.clone();
    for operation in item.as_object_mut().expect("a path item").values_mut() {
        let operation = operation.as_object_mut().expect("an operation");
        operation.remove("summary");
        operation.remove("description");
    }
    item
}

#[test]
fn the_published_snapshot_is_current() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/docs/api/openapi.json");
    let published = std::fs::read_to_string(path).expect("docs/api/openapi.json");
    let current = serde_json::to_string_pretty(&ours()).expect("JSON") + "\n";
    assert!(
        published == current,
        "docs/api/openapi.json is out of date: run s/openapi"
    );
}

#[tokio::test]
async fn every_documented_path_is_a_route() {
    let app = router(Arc::new(AppState::new(ReferenceStore::empty(), None)));
    let document = ours();
    let paths = document["paths"].as_object().expect("paths");
    for (path, item) in paths {
        for method in item.as_object().expect("a path item").keys() {
            let request = Request::builder()
                .method(method.to_uppercase().as_str())
                .uri(path.as_str())
                .header("host", "testserver")
                .header("content-type", "application/json")
                .body(Body::empty())
                .expect("request");
            let status = app
                .clone()
                .oneshot(request)
                .await
                .expect("response")
                .status();
            assert!(
                status != StatusCode::NOT_FOUND && status != StatusCode::METHOD_NOT_ALLOWED,
                "{method} {path}: {status}"
            );
        }
    }
}

#[test]
fn it_differs_from_the_pinned_servers_document_only_where_listed() {
    let ours = ours();
    let upstream = openapi::upstream();

    // Every upstream operation, shaped as upstream shapes it: only the
    // summaries and descriptions are opengrowth's own. `/` is the health
    // check, where upstream returns its document.
    let upstream_paths = upstream["paths"].as_object().expect("paths");
    for (path, item) in upstream_paths {
        let own = &ours["paths"][path];
        assert!(own.is_object(), "{path} is not documented");
        if path != "/" {
            assert_eq!(without_words(own), without_words(item), "{path}");
        }
    }
    // A path upstream lacks belongs to a reference upstream lacks.
    let upstream_prefixes: Vec<&str> = upstream_paths
        .keys()
        .filter_map(|path| path.trim_start_matches('/').split('/').next())
        .collect();
    for path in ours["paths"].as_object().expect("paths").keys() {
        if !upstream_paths.contains_key(path) {
            let prefix = path.trim_start_matches('/').split('/').next().unwrap_or("");
            assert!(
                !upstream_prefixes.contains(&prefix),
                "{path} is new for an upstream reference"
            );
        }
    }

    // Upstream's schemas, with these changes and no others.
    let own_schemas = &ours["components"]["schemas"];
    let mut expected = upstream["components"]["schemas"].clone();
    expected["CalculationEngine"]["properties"]["name"]["const"] = json!("opengrowth");
    expected["APIServer"]["properties"]["name"]["const"] = json!("opengrowth-server");
    expected["Provenance"]["properties"]["growth_reference"]["enum"] =
        own_schemas["Provenance"]["properties"]["growth_reference"]["enum"].clone();
    expected["Provenance"]["properties"]["reference_data"] =
        own_schemas["Provenance"]["properties"]["reference_data"].clone();
    expected["Provenance"]["required"]
        .as_array_mut()
        .expect("required")
        .push(json!("reference_data"));
    expected["ChartCoordinateRequest"]["properties"]["is_sds"]["description"] =
        own_schemas["ChartCoordinateRequest"]["properties"]["is_sds"]["description"].clone();
    expected["TableUsed"] = own_schemas["TableUsed"].clone();
    assert_eq!(own_schemas, &expected);

    // The reference list keeps upstream's six, in its order.
    let upstream_references =
        &upstream["components"]["schemas"]["Provenance"]["properties"]["growth_reference"]["enum"];
    let own_references = own_schemas["Provenance"]["properties"]["growth_reference"]["enum"]
        .as_array()
        .expect("enum");
    assert!(own_references.starts_with(upstream_references.as_array().expect("enum")));

    // opengrowth's identity throughout, never RCPCH's.
    assert_eq!(ours["info"]["title"], "opengrowth API");
    assert_eq!(ours["info"]["version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(
        ours["components"]["securitySchemes"]["bearerAuth"]["scheme"],
        "bearer"
    );
    let text = ours.to_string();
    for absent in [
        "api.rcpch.ac.uk",
        "Subscription-Key",
        "rcpch/rcpchgrowth-python",
        "rcpch/digital-growth-charts-server",
    ] {
        assert!(!text.contains(absent), "{absent}");
    }
}

/// Upstream's `test_openapi_documents_required_provenance_contract`, with
/// opengrowth's names and its `reference_data`.
#[test]
fn the_document_declares_the_provenance_contract() {
    let document = ours();
    let schemas = &document["components"]["schemas"];
    assert!(
        schemas["MeasurementObject"]["required"]
            .as_array()
            .expect("required")
            .contains(&json!("provenance"))
    );
    assert_eq!(
        schemas["Provenance"]["required"],
        json!([
            "growth_reference",
            "calculation_engine",
            "api_server",
            "reference_data"
        ])
    );
    assert_eq!(
        schemas["Provenance"]["properties"]["growth_reference"]["enum"],
        json!([
            "uk-who",
            "trisomy-21",
            "trisomy-21-aap",
            "turners-syndrome",
            "cdc",
            "who"
        ])
    );
    assert_eq!(
        schemas["APIServer"]["required"],
        json!(["name", "version", "commit"])
    );
    assert_eq!(
        schemas["APIServer"]["properties"]["name"]["const"],
        "opengrowth-server"
    );
    assert_eq!(
        schemas["CalculationEngine"]["properties"]["name"]["const"],
        "opengrowth"
    );
}
