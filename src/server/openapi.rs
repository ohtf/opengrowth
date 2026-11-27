// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-FileCopyrightText: Royal College of Paediatrics and Child Health
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The OpenAPI document for the HTTP API (`R15`, `R49`).
//!
//! The API is the pinned upstream server's, byte for byte, so the schemas are
//! that server's own, from its document as `s/goldens http_openapi` captured
//! it, with opengrowth's identity applied: the engine and server names, the
//! `reference_data` list in provenance, and the bearer-token scheme in place
//! of RCPCH's gateway key. The paths are generated from the router's
//! operations and the reference definitions, so a reference added there is
//! documented with no further change; upstream's document supplies only
//! their request examples. `tests/openapi.rs` checks that the result differs
//! from upstream's only where listed.
//!
//! The server does not serve the document; the documentation publishes it
//! (`spec/standard.md` section 7.4).

use serde_json::{Map, Value, json};

use super::{MID_PARENTAL_HEIGHT, Operation, response};
use crate::data::TableId;
use crate::references::{self, Definition};
use crate::vocabulary::GrowthReference;

/// The pinned server's document, as `conformance/generators/http_openapi.py`
/// captured it.
const UPSTREAM: &str = include_str!("../../conformance/expected/http-openapi.json");

/// Where a deployment serves the API, for the document's `servers`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Server {
    /// The base URL.
    pub url: String,
    /// What it is, if anything needs saying.
    pub description: Option<String>,
}

/// The servers the published document lists: `opengrowth serve` on the
/// reader's own computer, with its default address.
#[must_use]
pub fn default_servers() -> Vec<Server> {
    vec![Server {
        url: "http://127.0.0.1:8000".to_owned(),
        description: Some("opengrowth serve, on this computer".to_owned()),
    }]
}

/// The pinned server's OpenAPI document.
///
/// # Panics
///
/// If the captured document is not the JSON that `s/goldens` writes, which
/// `tests/openapi.rs` would show first.
#[must_use]
pub fn upstream() -> Value {
    let golden: Value = serde_json::from_str(UPSTREAM).expect("the captured document is JSON");
    golden["cases"]["document"].clone()
}

/// opengrowth's OpenAPI document, listing `servers`.
#[must_use]
pub fn document(servers: &[Server]) -> Value {
    let upstream = upstream();
    let mut paths = Map::new();
    paths.insert("/".to_owned(), root_item());
    for definition in references::ALL {
        for operation in Operation::ALL {
            let path = format!("/{}/{}", definition.api.prefix, operation.path());
            let item = reference_item(&upstream, definition, operation, &path);
            paths.insert(path, json!({ "post": item }));
        }
    }
    paths.insert(
        MID_PARENTAL_HEIGHT.to_owned(),
        json!({ "post": mid_parental_height_item(&upstream) }),
    );
    let servers: Vec<Value> = servers
        .iter()
        .map(|server| match &server.description {
            Some(description) => json!({ "url": server.url, "description": description }),
            None => json!({ "url": server.url }),
        })
        .collect();
    json!({
        "openapi": "3.1.0",
        "info": {
            "title": "opengrowth API",
            "summary": "SDS, centiles, and chart coordinates for children's growth measurements.",
            "description": "Calculates the SDS and centile of a child's height or length, weight, head circumference, or body mass index against a growth reference the caller chooses, and the coordinates of the reference's centile lines, with mid-parental height and fictional children for testing. It takes the same requests and returns the same responses as the RCPCH Digital Growth Charts API, apart from listed differences, and is not affiliated with RCPCH. opengrowth is pre-alpha and not for clinical use. Documentation: https://ohtf.github.io/opengrowth/",
            "version": crate::ENGINE_VERSION,
            "license": { "name": "AGPL-3.0-or-later", "identifier": "AGPL-3.0-or-later" },
        },
        "servers": servers,
        "paths": paths,
        "components": {
            "schemas": schemas(&upstream),
            "securitySchemes": {
                "bearerAuth": {
                    "type": "http",
                    "scheme": "bearer",
                    "description": "Required only when the server is started with AUTHORIZATION_KEY set: send `Authorization: Bearer <key>`. `/` never needs it.",
                },
            },
        },
        "security": [{ "bearerAuth": [] }, {}],
    })
}

/// `GET /`, the health check.
fn root_item() -> Value {
    json!({
        "get": {
            "tags": ["health"],
            "summary": "Health check",
            "description": "The server's name and version, with the commit it was built from in the `x-git-revision` header. It never needs a token. Unlike the RCPCH server, opengrowth does not return its OpenAPI document here; the documentation publishes it.",
            "operationId": "root__get",
            "security": [],
            "responses": {
                "200": {
                    "description": "The server is running.",
                    "headers": {
                        "x-git-revision": {
                            "description": "The commit the server was built from, or `unknown`.",
                            "schema": { "type": "string" },
                        },
                    },
                    "content": {
                        "application/json": {
                            "schema": {
                                "type": "object",
                                "properties": {
                                    "name": { "type": "string", "const": response::API_SERVER_NAME },
                                    "version": { "type": "string" },
                                    "detail": { "type": "string" },
                                },
                                "required": ["name", "version", "detail"],
                            },
                        },
                    },
                },
            },
        },
    })
}

/// One operation of a reference, shaped as upstream shapes its UK-WHO one,
/// with upstream's request examples for this path where it has them.
fn reference_item(
    upstream: &Value,
    definition: &Definition,
    operation: Operation,
    path: &str,
) -> Value {
    let template = format!("/uk-who/{}", operation.path());
    let mut item = upstream["paths"][&template]["post"].clone();
    let id = operation_id(definition, operation, path);
    item["tags"] = json!([definition.reference.as_str()]);
    item["summary"] = json!(summary(definition, operation));
    item["description"] = json!(description(definition, operation));
    item["operationId"] = json!(id);
    let examples = upstream["paths"]
        .get(path)
        .and_then(|own| {
            own["post"]["requestBody"]["content"]["application/json"]["schema"].get("examples")
        })
        .cloned();
    let schema = &mut item["requestBody"]["content"]["application/json"]["schema"];
    match (examples, schema.as_object_mut()) {
        (Some(examples), _) => schema["examples"] = examples,
        (None, Some(schema)) => {
            schema.remove("examples");
        }
        (None, None) => {}
    }
    if operation == Operation::FictionalChildData {
        item["responses"]["200"]["content"]["application/json"]["schema"]["title"] =
            json!(response_title(&id));
    }
    item
}

/// `POST /utilities/mid-parental-height`, upstream's, with opengrowth's words.
fn mid_parental_height_item(upstream: &Value) -> Value {
    let mut item = upstream["paths"][MID_PARENTAL_HEIGHT]["post"].clone();
    item["summary"] = json!("Mid-parental height");
    item["description"] = json!(
        "Calculates the mid-parental height from both parents' heights against UK-WHO, WHO, or CDC, as a height, an SDS, and a centile, with the chart lines at that SDS and either side of it."
    );
    item
}

/// FastAPI's operation ID: the function's name, then the path, with every
/// character that is not a letter, digit, or underscore made one, then the
/// method. Upstream names every reference's fictional-child function alike.
fn operation_id(definition: &Definition, operation: Operation, path: &str) -> String {
    let function = match operation {
        Operation::FictionalChildData => "fictional_child_data".to_owned(),
        _ => format!("{}_{}", definition.api.prefix, operation.path()).replace('-', "_"),
    };
    format!("{function}{}_post", path.replace(['/', '-'], "_"))
}

/// FastAPI's title for a response schema it generates: "Response", then
/// the operation ID's words, capitalised.
fn response_title(operation_id: &str) -> String {
    let words: Vec<String> = operation_id
        .split('_')
        .map(|word| {
            let mut chars = word.chars();
            chars.next().map_or_else(String::new, |first| {
                first.to_uppercase().chain(chars).collect()
            })
        })
        .collect();
    format!("Response {}", words.join(" "))
}

fn summary(definition: &Definition, operation: Operation) -> String {
    let what = match operation {
        Operation::Calculation => "one measurement",
        Operation::BulkCalculation => "up to 200 measurements",
        Operation::ChartCoordinates => "chart coordinates",
        Operation::FictionalChildData => "fictional child data",
    };
    format!("{}: {what}", definition.label)
}

fn description(definition: &Definition, operation: Operation) -> String {
    let label = definition.label;
    let mut text = match operation {
        Operation::Calculation => format!(
            "Calculates the SDS and centile of one measurement against the {label} reference, at the child's chronological and corrected ages, with provenance naming the reference and the tables read."
        ),
        Operation::BulkCalculation => format!(
            "Calculates up to 200 measurements of one child against the {label} reference. An observation that fails validation becomes an inline error in its place, and the others are still calculated."
        ),
        Operation::ChartCoordinates => format!(
            "The coordinates of the {label} reference's centile lines, for drawing a chart: a named collection of centiles, or custom centiles or SDS values."
        ),
        Operation::FictionalChildData => format!(
            "Measurements of a fictional child that follow an SDS line, optionally with drift and noise, calculated against the {label} reference, for testing and demonstration."
        ),
    };
    if let Some(only) = definition.api.only {
        let sex = match only.sex {
            crate::vocabulary::Sex::Female => "girls",
            crate::vocabulary::Sex::Male => "boys",
        };
        text.push_str(&format!(
            " The reference covers only {} in {sex}.",
            method_name(only.method)
        ));
    }
    text
}

fn method_name(method: crate::vocabulary::MeasurementMethod) -> &'static str {
    match method {
        crate::vocabulary::MeasurementMethod::Height => "height",
        crate::vocabulary::MeasurementMethod::Weight => "weight",
        crate::vocabulary::MeasurementMethod::Bmi => "BMI",
        crate::vocabulary::MeasurementMethod::Ofc => "head circumference",
    }
}

/// Upstream's schemas, with opengrowth's identity applied.
fn schemas(upstream: &Value) -> Value {
    let mut schemas = upstream["components"]["schemas"].clone();
    schemas["CalculationEngine"]["properties"]["name"]["const"] = json!(crate::ENGINE_NAME);
    schemas["APIServer"]["properties"]["name"]["const"] = json!(response::API_SERVER_NAME);

    let provenance = &mut schemas["Provenance"];
    let references: Vec<&str> = GrowthReference::ALL
        .iter()
        .map(|reference| reference.as_str())
        .collect();
    provenance["properties"]["growth_reference"]["enum"] = json!(references);
    provenance["properties"]["reference_data"] = json!({
        "type": "array",
        "items": { "$ref": "#/components/schemas/TableUsed" },
        "title": "Reference Data",
        "description": "Every reference table the calculation read, once each.",
    });
    if let Some(required) = provenance["required"].as_array_mut() {
        required.push(json!("reference_data"));
    }

    // Upstream's chart and mid-parental height responses carry no
    // provenance; opengrowth's carry it first, as its measurements do.
    for name in ["Centile_Data", "MidParentalHeightResponse"] {
        let schema = &mut schemas[name];
        let mut properties = Map::new();
        properties.insert(
            "provenance".to_owned(),
            json!({ "$ref": "#/components/schemas/Provenance" }),
        );
        if let Some(own) = schema["properties"].as_object() {
            properties.extend(own.clone());
        }
        schema["properties"] = Value::Object(properties);
        let mut required = vec![json!("provenance")];
        if let Some(own) = schema["required"].as_array() {
            required.extend(own.iter().cloned());
        }
        schema["required"] = Value::Array(required);
    }

    // Upstream declares each sex's lines as an object, but both servers send
    // `null` for the sex not asked for.
    for sex in ["male", "female"] {
        schemas["Sex"]["properties"][sex] = json!({
            "anyOf": [{ "$ref": "#/components/schemas/MeasurementMethod" }, { "type": "null" }],
        });
    }

    // Upstream calls it a "Boolean flag ... referring to centile_format".
    schemas["ChartCoordinateRequest"]["properties"]["is_sds"]["description"] = json!(
        "Whether the custom values in centile_format are SDS rather than centiles. Defaults to false."
    );

    let tables: Vec<&str> = TableId::ALL.iter().map(|table| table.as_str()).collect();
    schemas["TableUsed"] = json!({
        "properties": {
            "table": { "type": "string", "enum": tables, "title": "Table" },
            "file": { "type": "string", "title": "File" },
            "sha256": { "type": "string", "pattern": "^[0-9a-f]{64}$", "title": "Sha256" },
            "source": { "type": "string", "enum": ["embedded", "imported"], "title": "Source" },
        },
        "type": "object",
        "required": ["table", "file", "sha256", "source"],
        "title": "TableUsed",
        "description": "A reference table a calculation read: the file it came from, that file's checksum, and whether it was compiled in or imported.",
    });
    schemas
}
