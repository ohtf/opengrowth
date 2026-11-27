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
    for sex in ["male", "female"] {
        expected["Sex"]["properties"][sex] = json!({
            "anyOf": [{ "$ref": "#/components/schemas/MeasurementMethod" }, { "type": "null" }],
        });
    }
    for name in ["Centile_Data", "MidParentalHeightResponse"] {
        expected[name]["properties"]["provenance"] =
            json!({ "$ref": "#/components/schemas/Provenance" });
        let required = expected[name]["required"].as_array_mut().expect("required");
        required.insert(0, json!("provenance"));
    }
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
    // Upstream's chart and mid-parental height responses have none.
    for name in ["Centile_Data", "MidParentalHeightResponse"] {
        assert_eq!(schemas[name]["required"][0], "provenance", "{name}");
    }
}

/// A local `$ref`'s target, or the schema itself.
fn resolve<'a>(document: &'a Value, schema: &'a Value) -> &'a Value {
    match schema.get("$ref").and_then(Value::as_str) {
        Some(target) => {
            let name = target
                .strip_prefix("#/components/schemas/")
                .expect("a local reference");
            &document["components"]["schemas"][name]
        }
        None => schema,
    }
}

/// Whether the schema's `type`, if it has one, admits the value.
fn admits_type(schema: &Value, value: &Value) -> bool {
    let kinds: Vec<&str> = match schema.get("type") {
        Some(Value::String(kind)) => vec![kind.as_str()],
        Some(Value::Array(kinds)) => kinds.iter().filter_map(Value::as_str).collect(),
        _ => return true,
    };
    kinds.iter().any(|kind| match (*kind, value) {
        ("null", Value::Null)
        | ("boolean", Value::Bool(_))
        | ("string", Value::String(_))
        | ("number", Value::Number(_))
        | ("array", Value::Array(_))
        | ("object", Value::Object(_)) => true,
        ("integer", Value::Number(number)) => number.is_i64() || number.is_u64(),
        _ => false,
    })
}

/// Every place where `value` is not as `schema` describes it: a key it does
/// not declare, a required key missing, or a type, `enum`, or `const` it does
/// not allow, recursively.
fn mismatches(document: &Value, schema: &Value, value: &Value, at: &str, found: &mut Vec<String>) {
    let schema = resolve(document, schema);
    let alternatives = schema
        .get("anyOf")
        .or_else(|| schema.get("oneOf"))
        .and_then(Value::as_array);
    if let Some(alternatives) = alternatives {
        let mut first = None;
        for alternative in alternatives {
            let mut here = Vec::new();
            mismatches(document, alternative, value, at, &mut here);
            if here.is_empty() {
                return;
            }
            first.get_or_insert(here);
        }
        found.extend(first.unwrap_or_default());
        return;
    }
    if !admits_type(schema, value) {
        found.push(format!("{at}: {value} is not {}", schema["type"]));
        return;
    }
    if let Some(allowed) = schema.get("enum").and_then(Value::as_array)
        && !allowed.contains(value)
    {
        found.push(format!("{at}: {value} is not one of {}", schema["enum"]));
    }
    if let Some(constant) = schema.get("const")
        && constant != value
    {
        found.push(format!("{at}: {value} is not {constant}"));
    }
    match value {
        Value::Object(object) => {
            let properties = schema.get("properties").and_then(Value::as_object);
            let additional = schema
                .get("additionalProperties")
                .filter(|extra| extra.is_object());
            let open = properties.is_none() && additional.is_none();
            for (key, item) in object {
                let at = format!("{at}/{key}");
                match (
                    properties.and_then(|declared| declared.get(key)),
                    additional,
                ) {
                    (Some(property), _) => mismatches(document, property, item, &at, found),
                    (None, Some(extra)) => mismatches(document, extra, item, &at, found),
                    (None, None) if open => {}
                    (None, None) => found.push(format!("{at}: not declared")),
                }
            }
            let required = schema.get("required").and_then(Value::as_array);
            for key in required.into_iter().flatten().filter_map(Value::as_str) {
                if !object.contains_key(key) {
                    found.push(format!("{at}/{key}: required, and missing"));
                }
            }
        }
        Value::Array(items) => {
            if let Some(item_schema) = schema.get("items") {
                for (index, item) in items.iter().enumerate() {
                    mismatches(document, item_schema, item, &format!("{at}/{index}"), found);
                }
            }
        }
        _ => {}
    }
}

/// Every captured request in a golden file, wherever its cases are kept.
fn captured(node: &Value, into: &mut Vec<Value>) {
    match node {
        Value::Object(object)
            if object.contains_key("request_body") && object.contains_key("path") =>
        {
            into.push(node.clone());
        }
        Value::Object(object) => object.values().for_each(|child| captured(child, into)),
        Value::Array(items) => items.iter().for_each(|child| captured(child, into)),
        _ => {}
    }
}

/// The document describes everything the server sends. For every documented
/// operation, opengrowth's responses to requests captured from the pinned
/// server match the response schema, provenance included: a field the
/// server sends that the schema omits is a bug (`spec/provenance.md`).
#[cfg(feature = "embedded-data")]
#[tokio::test]
async fn every_response_matches_the_document() {
    let document = ours();
    let app = router(Arc::new(AppState::new(
        ReferenceStore::standard(None).expect("tables load"),
        None,
    )));
    let directory = concat!(env!("CARGO_MANIFEST_DIR"), "/conformance/expected");
    let mut files: Vec<_> = std::fs::read_dir(directory)
        .expect("goldens")
        .map(|entry| entry.expect("an entry").path())
        .filter(|file| {
            let name = file
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("");
            let other = [
                "http-cors.json",
                "http-openapi.json",
                "http-request-bodies.json",
            ];
            name.starts_with("http-") && !other.contains(&name)
        })
        .collect();
    files.sort();
    let mut cases = Vec::new();
    for file in &files {
        let text = std::fs::read_to_string(file).expect("a golden");
        let golden: Value = serde_json::from_str(&text).expect("JSON");
        captured(&golden["cases"], &mut cases);
    }

    let mut found = Vec::new();
    let mut checked = 0;
    for (path, item) in document["paths"].as_object().expect("paths") {
        let Some(operation) = item.get("post") else {
            continue;
        };
        let schema = &operation["responses"]["200"]["content"]["application/json"]["schema"];
        let sample: Vec<&Value> = cases
            .iter()
            .filter(|case| {
                case["path"] == path.as_str() && case["method"] == "POST" && case["status"] == 200
            })
            .collect();
        assert!(!sample.is_empty(), "no captured 200 response for {path}");
        checked += sample.len();
        for case in sample {
            let body = case["request_body"].as_str().expect("a body").to_owned();
            let request = Request::builder()
                .method("POST")
                .uri(path.as_str())
                .header("host", "testserver")
                .header("content-type", "application/json")
                .body(Body::from(body))
                .expect("request");
            let response = app.clone().oneshot(request).await.expect("response");
            assert_eq!(
                response.status(),
                StatusCode::OK,
                "{path}: {}",
                case["name"]
            );
            let bytes = http_body_util::BodyExt::collect(response.into_body())
                .await
                .expect("body")
                .to_bytes();
            let sent: Value = serde_json::from_slice(&bytes).expect("a JSON response");
            let at = format!("{path} ({})", case["name"]);
            mismatches(&document, schema, &sent, &at, &mut found);
        }
    }
    assert!(
        found.is_empty(),
        "{} mismatches in {checked} responses:\n{}",
        found.len(),
        found.join("\n")
    );
    eprintln!("{checked} responses match the document");
}
