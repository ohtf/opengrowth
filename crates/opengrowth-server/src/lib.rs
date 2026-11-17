// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The opengrowth HTTP API, compatible with the RCPCH Digital Growth Charts
//! API (`digital-growth-charts-server`).
//!
//! The goal is byte-identical response bodies outside `provenance`
//! (`spec/standard.md` section 4.2). Implemented so far: UK-WHO
//! `calculation` and `bulk-calculation`, bearer-token authorisation,
//! 404, 405, and trailing-slash redirects. Other operations return 501.
//!
//! The `/growth/v1` prefix belongs to the gateway, not this router.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod pyjson;
pub mod response;
pub mod validate;

use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use axum::body::{Body, Bytes};
use axum::extract::{Request, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use opengrowth_core::dates::{calendar_age, corrected_decimal_age};
use opengrowth_core::measurement::{MeasureError, MeasurementInput, measure};
use opengrowth_core::{
    CalcError, DataError, Gestation, GrowthReference, MeasurementMethod, ReferenceData, Sex,
    sds_for_measurement,
};
use opengrowth_data::ReferenceStore;
use serde::Serialize;
use serde_json::{Value, json};

use response::{
    Detail, EchoedGestation, InlineError, MeasurementObject, Provenance, RouteError, TableUsed,
};
use validate::{BirthFields, FieldError, LocItem, ObservationFields};

/// Most observations a bulk request may carry, as upstream.
pub const MAX_BULK_OBSERVATIONS: usize = 200;

/// Shared server state.
#[derive(Debug)]
pub struct AppState {
    /// Verified reference tables.
    pub store: ReferenceStore,
    /// If set, every request except `/` must send `Authorization: Bearer <key>`.
    pub authorization_key: Option<String>,
}

/// The application router.
pub fn router(state: Arc<AppState>) -> Router {
    let mut app = Router::new()
        .route("/", get(root))
        .route(
            "/uk-who/calculation",
            post(uk_who_calculation).fallback(method_not_allowed),
        )
        .route(
            "/uk-who/bulk-calculation",
            post(uk_who_bulk_calculation).fallback(method_not_allowed),
        );
    for path in NOT_YET_IMPLEMENTED {
        app = app.route(path, post(not_implemented).fallback(method_not_allowed));
    }
    app.fallback(not_found)
        .layer(middleware::from_fn_with_state(state.clone(), authorise))
        .with_state(state)
}

/// Serves the API until interrupted.
///
/// # Errors
///
/// If the address cannot be bound or the server fails.
pub async fn serve(address: SocketAddr, state: Arc<AppState>) -> std::io::Result<()> {
    let listener = tokio::net::TcpListener::bind(address).await?;
    axum::serve(listener, router(state))
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await
}

/// The other upstream operations, routed so they report 501 rather than 404.
const NOT_YET_IMPLEMENTED: &[&str] = &[
    "/uk-who/chart-coordinates",
    "/uk-who/fictional-child-data",
    "/turners-syndrome/calculation",
    "/turners-syndrome/bulk-calculation",
    "/turners-syndrome/chart-coordinates",
    "/turners-syndrome/fictional-child-data",
    "/trisomy-21/calculation",
    "/trisomy-21/bulk-calculation",
    "/trisomy-21/chart-coordinates",
    "/trisomy-21/fictional-child-data",
    "/trisomy-21-aap/calculation",
    "/trisomy-21-aap/bulk-calculation",
    "/trisomy-21-aap/chart-coordinates",
    "/trisomy-21-aap/fictional-child-data",
    "/cdc/calculation",
    "/cdc/bulk-calculation",
    "/cdc/chart-coordinates",
    "/cdc/fictional-child-data",
    "/who/calculation",
    "/who/bulk-calculation",
    "/who/chart-coordinates",
    "/who/fictional-child-data",
    "/utilities/mid-parental-height",
];

fn json_response<T: Serialize + ?Sized>(status: StatusCode, body: &T) -> Response {
    let mut response = Response::new(Body::from(pyjson::to_vec(body)));
    *response.status_mut() = status;
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );
    response
}

fn detail(status: StatusCode, message: &str) -> Response {
    json_response(status, &json!({ "detail": message }))
}

async fn authorise(State(state): State<Arc<AppState>>, request: Request, next: Next) -> Response {
    let open = request.uri().path() == "/";
    if let Some(key) = state.authorization_key.as_deref().filter(|k| !k.is_empty())
        && !open
    {
        let expected = format!("Bearer {key}");
        let sent = request
            .headers()
            .get(header::AUTHORIZATION)
            .map(HeaderValue::as_bytes);
        if sent != Some(expected.as_bytes()) {
            return detail(StatusCode::FORBIDDEN, "Forbidden");
        }
    }
    next.run(request).await
}

async fn root() -> Response {
    let mut response = json_response(
        StatusCode::OK,
        &json!({
            "name": response::API_SERVER_NAME,
            "version": opengrowth_core::ENGINE_VERSION,
            "detail": "The OpenAPI document is not served yet (R15).",
        }),
    );
    response
        .headers_mut()
        .insert("x-git-revision", HeaderValue::from_static(response::COMMIT));
    response
}

/// Starlette's `redirect_slashes`: a path that matches only with or without
/// its trailing slash is redirected with 307, whatever the method.
async fn not_found(request: Request) -> Response {
    let path = request.uri().path();
    if path != "/" && path.ends_with('/') {
        let trimmed = path.trim_end_matches('/');
        let known = matches!(trimmed, "/uk-who/calculation" | "/uk-who/bulk-calculation")
            || NOT_YET_IMPLEMENTED.contains(&trimmed);
        if known {
            let host = request
                .headers()
                .get(header::HOST)
                .and_then(|h| h.to_str().ok())
                .unwrap_or("localhost");
            let query = request
                .uri()
                .query()
                .map(|q| format!("?{q}"))
                .unwrap_or_default();
            let location = format!("http://{host}{trimmed}{query}");
            let mut response = Response::new(Body::empty());
            *response.status_mut() = StatusCode::TEMPORARY_REDIRECT;
            if let Ok(value) = HeaderValue::from_str(&location) {
                response.headers_mut().insert(header::LOCATION, value);
            }
            return response;
        }
    }
    detail(StatusCode::NOT_FOUND, "Not Found")
}

async fn method_not_allowed() -> Response {
    let mut response = detail(StatusCode::METHOD_NOT_ALLOWED, "Method Not Allowed");
    response
        .headers_mut()
        .insert(header::ALLOW, HeaderValue::from_static("POST"));
    response
}

async fn not_implemented(request: Request) -> Response {
    detail(
        StatusCode::NOT_IMPLEMENTED,
        &format!(
            "{} is not implemented in opengrowth yet.",
            request.uri().path()
        ),
    )
}

fn validation_failed(errors: Vec<FieldError>) -> Response {
    json_response(StatusCode::UNPROCESSABLE_ENTITY, &Detail { detail: errors })
}

fn route_failed(error: RouteError) -> Response {
    json_response(
        StatusCode::UNPROCESSABLE_ENTITY,
        &Detail { detail: [error] },
    )
}

fn data_unavailable(error: &DataError) -> Response {
    detail(
        StatusCode::SERVICE_UNAVAILABLE,
        &format!("{error}. Reference tables must be imported with `opengrowth data import`."),
    )
}

/// FastAPI's body handling: an empty body is a missing body, and JSON is
/// decoded with CPython's `json` module semantics.
fn parse_body(bytes: &Bytes) -> Result<Value, Box<Response>> {
    if bytes.is_empty() {
        return Err(Box::new(validation_failed(vec![FieldError {
            kind: "missing",
            loc: vec!["body".into()],
            msg: "Field required".to_owned(),
            input: Value::Null,
            ctx: None,
        }])));
    }
    serde_json::from_slice(bytes).map_err(|serde_error| {
        let text = String::from_utf8_lossy(bytes);
        let (message, position) = match pyjson::python_decode_error(&text) {
            Some(error) => (error.message, error.position),
            // CPython would accept this (NaN, Infinity, lone surrogates); opengrowth does not.
            None => (serde_error.to_string(), 0),
        };
        Box::new(validation_failed(vec![FieldError {
            kind: "json_invalid",
            loc: vec!["body".into(), LocItem::Index(position)],
            msg: "JSON decode error".to_owned(),
            input: json!({}),
            ctx: Some(json!({ "error": message })),
        }]))
    })
}

/// A route-level failure: an error to report, or a response to send as is.
enum Rejection {
    Report(RouteError),
    Respond(Box<Response>),
}

/// The upstream `validate_observation_value`: refuses a measurement more
/// than 8 SD (15 SD for BMI) from the mean at corrected age, or with no
/// reference data at that age.
fn validate_observation_value(
    store: &ReferenceStore,
    reference: GrowthReference,
    birth: &BirthFields,
    observation: &ObservationFields,
) -> Result<(Gestation, Sex, MeasurementMethod), Rejection> {
    let report = |msg: String| Rejection::Report(RouteError::value_error(msg, "observation_value"));
    // Upstream computes `(weeks * 7) + days` with whatever it was sent, so a
    // null reaches Python's arithmetic. The TypeError text is its message.
    let (weeks, days) = match (birth.gestation_weeks, birth.gestation_days) {
        (None, _) => {
            return Err(report(
                "unsupported operand type(s) for *: 'NoneType' and 'int'".to_owned(),
            ));
        }
        (Some(_), None) => {
            return Err(report(
                "unsupported operand type(s) for +: 'int' and 'NoneType'".to_owned(),
            ));
        }
        (Some(w), Some(d)) => (w, d),
    };
    let gestation = Gestation::new(
        u8::try_from(weeks).expect("validated 22 to 44"),
        u8::try_from(days).expect("validated 0 to 6"),
    )
    .expect("validated range");
    let sex: Sex = birth.sex.parse().expect("validated literal");
    let method: MeasurementMethod = birth.measurement_method.parse().expect("validated literal");
    let value = observation.observation_value;

    let age = corrected_decimal_age(birth.birth_date, observation.observation_date, gestation)
        .map_err(|e| report(e.to_string()))?;
    let sds = match sds_for_measurement(store, reference, age, method, value, sex) {
        Ok(result) => result.sds,
        Err(CalcError::UkWhoAbsent(absence)) => return Err(report(absence.to_string())),
        Err(CalcError::Data(error)) => {
            return Err(Rejection::Respond(Box::new(data_unavailable(&error))));
        }
        // Upstream gets a very negative real SDS, a complex number (500), or a
        // ZeroDivisionError (500) here, depending on L. Every case is reported
        // as below the limit, which is what upstream says whenever it answers.
        Err(CalcError::InvalidObservation(v)) if v == f64::INFINITY => f64::INFINITY,
        Err(CalcError::InvalidObservation(_)) => f64::NEG_INFINITY,
        Err(other) => return Err(report(other.to_string())),
    };
    let calendar = calendar_age(birth.birth_date, observation.observation_date)
        .map_err(|e| report(e.to_string()))?;
    let boy_girl = if sex == Sex::Male { "boy" } else { "girl" };
    let age_phrase = if calendar == "Birth date" {
        "at birth".to_owned()
    } else {
        format!("of {calendar}")
    };
    let value_text = opengrowth_core::pyfmt::python_repr(value);
    let (name, units) = match method {
        MeasurementMethod::Height => ("height", "cm"),
        MeasurementMethod::Ofc => ("Head circumference", "cm"),
        MeasurementMethod::Weight => ("weight", "kg"),
        MeasurementMethod::Bmi => {
            if sds < -15.0 {
                return Err(report(
                    "Body mass index cannot be less than -15 SD.".to_owned(),
                ));
            }
            if sds > 15.0 {
                return Err(report(
                    "Body mass index cannot be more than +15 SD.".to_owned(),
                ));
            }
            return Ok((gestation, sex, method));
        }
    };
    if sds < -8.0 {
        return Err(report(format!(
            "A {name} of {value_text} {units} in a {boy_girl} {age_phrase} is less than -8 SD. Please recheck the measurement and date of birth."
        )));
    }
    if sds > 8.0 {
        return Err(report(format!(
            "A {name} of {value_text} {units} in a {boy_girl} {age_phrase} is more than +8 SD. Please recheck the measurement and date of birth."
        )));
    }
    Ok((gestation, sex, method))
}

/// Calculates one observation and renders it, or the reason it failed.
fn calculate_one(
    state: &AppState,
    reference: GrowthReference,
    birth: &BirthFields,
    observation: &ObservationFields,
) -> Result<Vec<u8>, Rejection> {
    let (gestation, sex, method) =
        validate_observation_value(&state.store, reference, birth, observation)?;
    let input = MeasurementInput {
        reference,
        birth_date: birth.birth_date,
        observation_date: observation.observation_date,
        gestation,
        sex,
        method,
        observation_value: observation.observation_value,
    };
    let measurement = match measure(&state.store, &input) {
        Ok(m) => m,
        Err(MeasureError::Data(error)) => {
            return Err(Rejection::Respond(Box::new(data_unavailable(&error))));
        }
        Err(MeasureError::Raised(message)) => {
            return Err(Rejection::Report(RouteError::value_error(
                message,
                "calculation_error",
            )));
        }
        Err(error @ MeasureError::ReferenceNotImplemented(_)) => {
            return Err(Rejection::Respond(Box::new(detail(
                StatusCode::NOT_IMPLEMENTED,
                &error.to_string(),
            ))));
        }
    };
    let provenance = Provenance::new(
        reference.as_str(),
        tables_used(&state.store, &measurement.tables_read),
    );
    let gestational_age_text = measurement
        .measurement_dates
        .corrected_gestational_age
        .map(|(w, d)| format!("{w} + {d} weeks"))
        .unwrap_or_default();
    let object = MeasurementObject::new(
        &measurement,
        observation,
        EchoedGestation {
            weeks: birth.gestation_weeks,
            days: birth.gestation_days,
        },
        provenance,
        &gestational_age_text,
    );
    Ok(pyjson::to_vec(&object))
}

fn tables_used(store: &ReferenceStore, tables: &[opengrowth_core::TableId]) -> Vec<TableUsed> {
    tables
        .iter()
        .filter_map(|&id| store.file_for(id))
        .map(|file| TableUsed {
            table: file.source.id.as_str(),
            file: file.source.file,
            sha256: file.source.sha256,
        })
        .collect()
}

fn raw_json(status: StatusCode, bytes: Vec<u8>) -> Response {
    let mut response = Response::new(Body::from(bytes));
    *response.status_mut() = status;
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );
    response
}

async fn uk_who_calculation(State(state): State<Arc<AppState>>, body: Bytes) -> Response {
    let value = match parse_body(&body) {
        Ok(v) => v,
        Err(response) => return *response,
    };
    let request = match validate::measurement_request(&value) {
        Ok(r) => r,
        Err(errors) => return validation_failed(errors),
    };
    match calculate_one(
        &state,
        GrowthReference::UkWho,
        &request.birth,
        &request.observation,
    ) {
        Ok(bytes) => raw_json(StatusCode::OK, bytes),
        Err(Rejection::Report(error)) => route_failed(error),
        Err(Rejection::Respond(response)) => *response,
    }
}

async fn uk_who_bulk_calculation(State(state): State<Arc<AppState>>, body: Bytes) -> Response {
    let value = match parse_body(&body) {
        Ok(v) => v,
        Err(response) => return *response,
    };
    let request = match validate::bulk_measurement_request(&value) {
        Ok(r) => r,
        Err(errors) => return validation_failed(errors),
    };
    if request.observations.is_empty() {
        return route_failed(RouteError::value_error(
            "At least one observation is required for bulk calculation.",
            "observations",
        ));
    }
    if request.observations.len() > MAX_BULK_OBSERVATIONS {
        return route_failed(RouteError::value_error(
            format!("Number of observations exceeds maximum allowed ({MAX_BULK_OBSERVATIONS})."),
            "observations",
        ));
    }
    // Each result is rendered on its own and spliced in, so a measurement is
    // serialised once, by the same code as the single endpoint.
    let mut out = Vec::with_capacity(request.observations.len() * 4700 + 16);
    out.extend_from_slice(b"{\"results\":[");
    for (i, observation) in request.observations.iter().enumerate() {
        if i > 0 {
            out.push(b',');
        }
        match calculate_one(&state, GrowthReference::UkWho, &request.birth, observation) {
            Ok(bytes) => out.extend_from_slice(&bytes),
            Err(Rejection::Report(error)) => {
                out.extend_from_slice(&pyjson::to_vec(&InlineError::from(error)));
            }
            Err(Rejection::Respond(response)) => return *response,
        }
    }
    out.extend_from_slice(b"]}");
    raw_json(StatusCode::OK, out)
}

impl IntoResponse for Rejection {
    fn into_response(self) -> Response {
        match self {
            Self::Report(error) => route_failed(error),
            Self::Respond(response) => *response,
        }
    }
}

/// Whether the UK-WHO tables are all present, for a warning at start-up.
#[must_use]
pub fn uk_who_ready(store: &ReferenceStore) -> bool {
    use opengrowth_core::TableId;
    [
        TableId::Uk90Preterm,
        TableId::Who2006Infant,
        TableId::Who2006Child,
        TableId::Uk90Child,
    ]
    .iter()
    .all(|&id| {
        store
            .table(id, MeasurementMethod::Weight, Sex::Male)
            .is_ok()
    })
}
