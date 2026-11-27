// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The opengrowth HTTP API, compatible with the RCPCH Digital Growth Charts
//! API (`digital-growth-charts-server`).
//!
//! The goal is byte-identical response bodies outside `provenance`
//! (`spec/standard.md` section 4.2). Every upstream operation is implemented:
//! four for each reference in [`references::ALL`], under the route prefix
//! its definition gives, and the mid-parental height utility. Around them
//! are bearer-token authorisation, CORS ([`cors`]), 404, 405, and
//! trailing-slash redirects.
//!
//! The `/growth/v1` prefix belongs to the gateway, not this router.

pub mod cors;
pub mod openapi;
pub mod pyjson;
pub mod response;
pub mod validate;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use crate::advice::CentileFormat;
use crate::chart::{CentileSpec, ChartError, chart};
use crate::dates::{calendar_age, corrected_decimal_age};
use crate::fictional_child::{
    FictionalChildError, FictionalChildParams, IntervalUnit, fictional_child, raw_measurement,
    schedule,
};
use crate::measurement::{MeasureError, Measurement, MeasurementInput, measure};
use crate::mid_parental_height::{adult_age, expected_range, mid_parental_sds};
use crate::provenance::{Provenance, Record};
use crate::references;
use crate::round::{PyFloat, numpy_round, python_round};
use crate::sds::measurement_from_sds;
use crate::store::ReferenceStore;
use crate::{
    CalcError, DataError, Gestation, GrowthReference, MeasurementMethod, ReferenceData, Sex,
    normal, sds_for_measurement,
};
use axum::Router;
use axum::body::{Body, Bytes};
use axum::extract::{DefaultBodyLimit, Request, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use serde::Serialize;
use serde_json::{Value, json};

use response::{
    CentileData, ChartResponse, Detail, EchoedGestation, InlineError, MeasurementObject,
    MidParentalHeightResponse, RouteError,
};
use validate::{BirthFields, CentileFormatRequest, FieldError, LocItem, ObservationFields};

/// Most observations a bulk request may carry, as upstream.
pub const MAX_BULK_OBSERVATIONS: usize = 200;

/// Shared server state.
#[derive(Debug)]
pub struct AppState {
    /// Verified reference tables.
    pub store: ReferenceStore,
    /// If set, every request except `/` must send `Authorization: Bearer <key>`.
    pub authorization_key: Option<String>,
    /// Rendered responses for the named chart formats, calculated on first
    /// request. Upstream serves these from files; opengrowth calculates them
    /// with the same engine and tables as its measurements.
    chart_cache: Mutex<HashMap<ChartKey, Arc<Vec<u8>>>>,
    /// Where fictional children's measurement noise comes from.
    noise: NoiseSource,
    /// Which origins browsers may call the API from.
    pub cors: cors::CorsPolicy,
}

type ChartKey = (GrowthReference, CentileFormat, MeasurementMethod, Sex);

impl AppState {
    /// State serving `store`, optionally requiring a bearer token.
    #[must_use]
    pub fn new(store: ReferenceStore, authorization_key: Option<String>) -> Self {
        Self {
            store,
            authorization_key,
            chart_cache: Mutex::new(HashMap::new()),
            noise: NoiseSource::Random,
            cors: cors::CorsPolicy::AnyOrigin,
        }
    }

    /// Makes every noise draw for fictional children `fraction` of the way
    /// across its range, so that noisy series are reproducible. For the
    /// conformance tests, which capture upstream with the same fixed draw.
    #[must_use]
    pub fn with_fixed_noise(mut self, fraction: f64) -> Self {
        self.noise = NoiseSource::Fixed(fraction);
        self
    }

    /// Sets which origins browsers may call the API from.
    #[must_use]
    pub fn with_cors(mut self, policy: cors::CorsPolicy) -> Self {
        self.cors = policy;
        self
    }
}

/// Where fictional children's measurement noise comes from.
#[derive(Debug, Clone, Copy)]
enum NoiseSource {
    /// A fresh pseudo-random stream for each request.
    Random,
    /// Every draw is this fraction of the way across its range.
    Fixed(f64),
}

impl NoiseSource {
    /// A uniform source on `[0, 1)`, standing in for Python's `random.random()`.
    fn uniform(self) -> Box<dyn FnMut() -> f64> {
        match self {
            Self::Fixed(fraction) => Box::new(move || fraction),
            Self::Random => {
                // SplitMix64, seeded from the standard library's randomly
                // keyed hasher: plenty for simulated measurement error, and
                // no dependency. Not for anything that must be unpredictable.
                let mut state = std::hash::BuildHasher::hash_one(
                    &std::collections::hash_map::RandomState::new(),
                    (),
                );
                Box::new(move || {
                    state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
                    let mut z = state;
                    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
                    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
                    z ^= z >> 31;
                    (z >> 11) as f64 / (1_u64 << 53) as f64
                })
            }
        }
    }
}

/// The operations upstream serves for every reference, each at
/// `/{prefix}/{path}`, where the prefix is the reference definition's.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Operation {
    Calculation,
    BulkCalculation,
    ChartCoordinates,
    FictionalChildData,
}

impl Operation {
    const ALL: [Self; 4] = [
        Self::Calculation,
        Self::BulkCalculation,
        Self::ChartCoordinates,
        Self::FictionalChildData,
    ];

    /// The path after the reference's prefix.
    const fn path(self) -> &'static str {
        match self {
            Self::Calculation => "calculation",
            Self::BulkCalculation => "bulk-calculation",
            Self::ChartCoordinates => "chart-coordinates",
            Self::FictionalChildData => "fictional-child-data",
        }
    }

    async fn handle(
        self,
        state: Arc<AppState>,
        body: RequestBody,
        reference: GrowthReference,
    ) -> Response {
        match self {
            Self::Calculation => calculation(state, body, reference).await,
            Self::BulkCalculation => bulk_calculation(state, body, reference).await,
            Self::ChartCoordinates => chart_coordinates(state, body, reference).await,
            Self::FictionalChildData => fictional_child_data(state, body, reference).await,
        }
    }
}

/// The mid-parental height utility, the one operation that belongs to no
/// reference.
const MID_PARENTAL_HEIGHT: &str = "/utilities/mid-parental-height";

/// The largest request body the server reads: 2 MiB, axum's default, made
/// explicit so that an upgrade cannot change it. A bulk request of 200
/// observations is about 12 KB. A larger body gets `413 Payload Too Large`,
/// where the RCPCH server reads a body of any size (`spec/standard.md`
/// section 7.4).
pub const MAX_REQUEST_BODY_BYTES: usize = 2 * 1024 * 1024;

/// Whether `path` is one of upstream's operations.
fn is_operation(path: &str) -> bool {
    if path == MID_PARENTAL_HEIGHT {
        return true;
    }
    let Some((prefix, operation)) = path.strip_prefix('/').and_then(|p| p.split_once('/')) else {
        return false;
    };
    references::ALL.iter().any(|d| d.api.prefix == prefix)
        && Operation::ALL.iter().any(|o| o.path() == operation)
}

/// The application router.
pub fn router(state: Arc<AppState>) -> Router {
    let mut app = Router::new().route("/", get(root));
    for definition in references::ALL {
        let reference = definition.reference;
        for operation in Operation::ALL {
            app = app.route(
                &format!("/{}/{}", definition.api.prefix, operation.path()),
                post(
                    move |State(state): State<Arc<AppState>>, headers: HeaderMap, body: Bytes| {
                        operation.handle(state, RequestBody::new(&headers, body), reference)
                    },
                )
                .fallback(method_not_allowed),
            );
        }
    }
    app = app.route(
        MID_PARENTAL_HEIGHT,
        post(utilities_mid_parental_height).fallback(method_not_allowed),
    );
    let api = app
        .fallback(not_found)
        .layer(DefaultBodyLimit::max(MAX_REQUEST_BODY_BYTES))
        .with_state(state.clone());
    // The middleware wraps the whole router, as upstream's wraps its app, so
    // that method routing cannot add `Allow` to what the middleware returns.
    // CORS is outermost, so preflights are answered before authorisation.
    Router::new()
        .fallback_service(api)
        .layer(middleware::from_fn_with_state(state.clone(), authorise))
        .layer(middleware::from_fn_with_state(state, cors::cors))
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
            .map_or(&[][..], HeaderValue::as_bytes);
        if !same_secret(sent, expected.as_bytes()) {
            return detail(StatusCode::FORBIDDEN, "Forbidden");
        }
    }
    next.run(request).await
}

/// Whether two byte strings are equal, in a time that depends only on their
/// lengths, so that how long a refusal takes does not reveal how much of a
/// key was right.
fn same_secret(sent: &[u8], expected: &[u8]) -> bool {
    if sent.len() != expected.len() {
        return false;
    }
    let difference = sent
        .iter()
        .zip(expected)
        .fold(0_u8, |difference, (a, b)| difference | (a ^ b));
    std::hint::black_box(difference) == 0
}

/// The open health check. Unlike the pinned server's `/`, it does not return
/// the OpenAPI document (`spec/standard.md` section 7.4).
async fn root() -> Response {
    let mut response = json_response(
        StatusCode::OK,
        &json!({
            "name": response::API_SERVER_NAME,
            "version": crate::ENGINE_VERSION,
            "detail": "The API does not serve its OpenAPI document; it will be published with the documentation (R30).",
        }),
    );
    response.headers_mut().insert(
        "x-git-revision",
        HeaderValue::from_static(crate::provenance::COMMIT),
    );
    response
}

/// Starlette's `redirect_slashes`: a path that matches only with or without
/// its trailing slash is redirected with 307, whatever the method.
async fn not_found(request: Request) -> Response {
    let path = request.uri().path();
    if path != "/" && path.ends_with('/') {
        let trimmed = path.trim_end_matches('/');
        if is_operation(trimmed) {
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

/// A request's body, and the `Content-Type` that says how to read it.
#[derive(Debug)]
struct RequestBody {
    content_type: Option<HeaderValue>,
    bytes: Bytes,
}

impl RequestBody {
    fn new(headers: &HeaderMap, bytes: Bytes) -> Self {
        Self {
            content_type: headers.get(header::CONTENT_TYPE).cloned(),
            bytes,
        }
    }
}

/// Whether FastAPI reads a body with this `Content-Type` as JSON: when it
/// is absent or empty, or when Python's `email.message` gives it the main
/// type `application` and a subtype `json` or ending in `+json`. A browser
/// sends `text/plain`, form posts, and file uploads to another site without
/// a CORS preflight, so not reading those as JSON keeps every cross-site
/// call behind the CORS policy: FastAPI's fix for CVE-2021-32677
/// (`spec/queries.md` Q16).
fn labelled_json(content_type: Option<&HeaderValue>) -> bool {
    let Some(value) = content_type else {
        return true;
    };
    // Starlette decodes header values as Latin-1.
    let value: String = value.as_bytes().iter().copied().map(char::from).collect();
    if value.is_empty() {
        return true;
    }
    // `get_content_type`: the part before any `;`, stripped as `str.strip`
    // strips, and lower-cased. Without exactly one `/` it is `text/plain`.
    let media_type = value
        .split(';')
        .next()
        .unwrap_or("")
        .trim_matches(python_space)
        .to_lowercase();
    match media_type.split_once('/') {
        Some((main, sub)) if !sub.contains('/') => {
            main == "application" && (sub == "json" || sub.ends_with("+json"))
        }
        _ => false,
    }
}

/// Whether Python's `str.isspace` holds for a Latin-1 character.
fn python_space(c: char) -> bool {
    matches!(c, '\t'..='\r' | '\u{1c}'..='\u{1f}' | ' ' | '\u{85}' | '\u{a0}')
}

/// FastAPI's body handling (`fastapi.routing.get_request_handler`): an
/// empty body is a missing body; a body not labelled as JSON
/// ([`labelled_json`]) goes to the request model undecoded, which refuses
/// it; and a JSON body is decoded and parsed as CPython's `json.loads` does.
fn parse_body(body: &RequestBody) -> Result<Value, Box<Response>> {
    let bytes = &body.bytes;
    if bytes.is_empty() {
        return Err(Box::new(validation_failed(vec![FieldError {
            kind: "missing",
            loc: vec!["body".into()],
            msg: "Field required".to_owned(),
            input: Value::Null,
            ctx: None,
        }])));
    }
    if !labelled_json(body.content_type.as_ref()) {
        // Upstream shows the body as UTF-8 text, and fails with a 500 where
        // it is not UTF-8; opengrowth shows it with replacement characters
        // (`spec/standard.md` section 6.3).
        return Err(Box::new(validation_failed(vec![FieldError {
            kind: "model_attributes_type",
            loc: vec!["body".into()],
            msg: "Input should be a valid dictionary or object to extract fields from".to_owned(),
            input: Value::String(String::from_utf8_lossy(bytes).into_owned()),
            ctx: None,
        }])));
    }
    let Some(text) = pyjson::decode_json_bytes(bytes) else {
        return Err(Box::new(detail(
            StatusCode::BAD_REQUEST,
            "There was an error parsing the body",
        )));
    };
    serde_json::from_str(&text).map_err(|serde_error| {
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
    Report(Box<RouteError>),
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
    let report = |msg: String| {
        Rejection::Report(Box::new(RouteError::value_error(msg, "observation_value")))
    };
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
        Err(CalcError::Absent(absence)) => return Err(report(absence.to_string())),
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
    let value_text = crate::pyfmt::python_repr(value);
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
    let api = references::definition(reference).api;
    // Upstream's Turner routes check these before anything else.
    if let Some(only) = api.only {
        let refuse = |msg: &str, field| {
            Err(Rejection::Report(Box::new(RouteError::value_error(
                msg, field,
            ))))
        };
        if birth.sex != only.sex.as_str() {
            return refuse(only.calculation_sex, "sex");
        }
        if birth.measurement_method != only.method.as_str() {
            return refuse(only.calculation_method, "measurement_method");
        }
    }
    // Upstream's AAP Trisomy 21 routes pass no bone age to the calculation.
    let without_bone_age;
    let observation = if !api.bone_age {
        without_bone_age = ObservationFields {
            bone_age: None,
            bone_age_type: None,
            bone_age_sds: None,
            bone_age_centile: None,
            bone_age_text: None,
            ..observation.clone()
        };
        &without_bone_age
    } else {
        observation
    };
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
            return Err(Rejection::Report(Box::new(RouteError::value_error(
                message,
                "calculation_error",
            ))));
        }
    };
    Ok(render_measurement(
        &state.store,
        &measurement,
        observation,
        EchoedGestation {
            weeks: birth.gestation_weeks,
            days: birth.gestation_days,
        },
    ))
}

/// One `MeasurementObject`, with the provenance the engine recorded.
fn render_measurement(
    store: &ReferenceStore,
    measurement: &Measurement,
    observation: &ObservationFields,
    gestation: EchoedGestation,
) -> Vec<u8> {
    let gestational_age_text = measurement
        .measurement_dates
        .corrected_gestational_age
        .map(|(w, d)| format!("{w} + {d} weeks"))
        .unwrap_or_default();
    let object = MeasurementObject::new(
        measurement,
        observation,
        gestation,
        record(store, &measurement.provenance),
        &gestational_age_text,
    );
    pyjson::to_vec(&object)
}

/// The provenance the engine recorded, as the server writes it: with the
/// file behind each table read, and the server's own identity.
fn record(store: &ReferenceStore, provenance: &Provenance) -> Record {
    Record::new(
        provenance,
        |id| store.table_used(id),
        Some(response::API_SERVER),
    )
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

async fn calculation(
    state: Arc<AppState>,
    body: RequestBody,
    reference: GrowthReference,
) -> Response {
    let value = match parse_body(&body) {
        Ok(v) => v,
        Err(response) => return *response,
    };
    let request = match validate::measurement_request(&value) {
        Ok(r) => r,
        Err(errors) => return validation_failed(errors),
    };
    match calculate_one(&state, reference, &request.birth, &request.observation) {
        Ok(bytes) => raw_json(StatusCode::OK, bytes),
        Err(Rejection::Report(error)) => route_failed(*error),
        Err(Rejection::Respond(response)) => *response,
    }
}

async fn bulk_calculation(
    state: Arc<AppState>,
    body: RequestBody,
    reference: GrowthReference,
) -> Response {
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
        match calculate_one(&state, reference, &request.birth, observation) {
            Ok(bytes) => out.extend_from_slice(&bytes),
            Err(Rejection::Report(error)) => {
                out.extend_from_slice(&pyjson::to_vec(&InlineError::from(*error)));
            }
            Err(Rejection::Respond(response)) => return *response,
        }
    }
    out.extend_from_slice(b"]}");
    raw_json(StatusCode::OK, out)
}

async fn fictional_child_data(
    state: Arc<AppState>,
    body: RequestBody,
    reference: GrowthReference,
) -> Response {
    let value = match parse_body(&body) {
        Ok(v) => v,
        Err(response) => return *response,
    };
    let request = match validate::fictional_child_request(&value) {
        Ok(r) => r,
        Err(errors) => return validation_failed(errors),
    };
    let label = references::definition(reference).api.fictional_child_name;
    match fictional_child_response(&state, reference, label, &request) {
        Ok(bytes) => raw_json(StatusCode::OK, bytes),
        Err(Rejection::Report(error)) => route_failed(*error),
        Err(Rejection::Respond(response)) => *response,
    }
}

/// The upstream route, which turns any exception into one 422. Upstream
/// accepts `null` for every optional field and then fails, or does not,
/// wherever Python first uses the value; each case is reproduced here.
fn fictional_child_response(
    state: &AppState,
    reference: GrowthReference,
    label: &str,
    request: &validate::FictionalChildRequest,
) -> Result<Vec<u8>, Rejection> {
    if let Some(only) = references::definition(reference).api.only {
        let refuse = |msg: &str| {
            Err(Rejection::Report(Box::new(RouteError::request(
                msg.to_owned(),
            ))))
        };
        if request.sex != only.sex.as_str() {
            return refuse(only.fictional_child_sex);
        }
        if request.measurement_method != only.method.as_str() {
            return refuse(only.fictional_child_method);
        }
    }
    let failed = || {
        Rejection::Report(Box::new(RouteError::request(format!(
            "Not possible to create {label} fictional child data."
        ))))
    };
    let rejected = |error: FictionalChildError| match error {
        FictionalChildError::Invalid(_) => failed(),
        FictionalChildError::Data(error) => Rejection::Respond(Box::new(data_unavailable(&error))),
    };
    // Each of these is always used, so a null is always a TypeError.
    let (Some(start_age), Some(end_age), Some(weeks), Some(unit), Some(interval_number)) = (
        request.start_chronological_age,
        request.end_age,
        request.gestation_weeks,
        request.measurement_interval_type.as_deref(),
        request.measurement_interval_number,
    ) else {
        return Err(failed());
    };
    // Below 40 weeks the gestation correction uses the days, and drift adds
    // `drift_range / cycles` to the SDS.
    let correction_needs_days = weeks < 40 && request.gestation_days.is_none();
    let drift_needs_values =
        request.drift && (request.drift_range.is_none() || request.start_sds.is_none());
    if correction_needs_days || drift_needs_values {
        return Err(failed());
    }
    let params = FictionalChildParams {
        reference,
        method: request
            .measurement_method
            .parse()
            .expect("validated literal"),
        sex: request.sex.parse().expect("validated literal"),
        start_age,
        end_age,
        gestation: Gestation::new(
            u8::try_from(weeks).expect("validated 22 to 44"),
            u8::try_from(request.gestation_days.unwrap_or(0)).expect("validated 0 to 6"),
        )
        .expect("validated range"),
        interval_unit: IntervalUnit::from_name(unit).expect("validated literal"),
        interval_number,
        start_sds: request.start_sds.unwrap_or(0.0),
        drift_range: request.drift_range.filter(|_| request.drift),
        noise_range: request.noise_range.filter(|_| request.noise),
    };

    // A null SDS makes every lookup fail quietly, so there is never a
    // measurement; a null gestation_days or noise_range fails at the first
    // measurement. Either way the schedule can still fail on its own.
    let fails_at_first_measurement =
        request.gestation_days.is_none() || (request.noise && request.noise_range.is_none());
    if request.start_sds.is_none() || fails_at_first_measurement {
        for step in schedule(&params).map_err(rejected)? {
            let step = step.map_err(rejected)?;
            if request.start_sds.is_some()
                && raw_measurement(&state.store, &params, &step)
                    .map_err(rejected)?
                    .is_some()
            {
                return Err(failed());
            }
        }
        return Ok(b"[]".to_vec());
    }

    let mut uniform = state.noise.uniform();
    let measurements = fictional_child(&state.store, &params, &mut *uniform).map_err(rejected)?;
    // A measurement dated before birth has no corrected age. Upstream's
    // response model rejects it, which escapes as a 500; this returns the
    // route's own 422 (`spec/standard.md` section 6.3).
    if measurements
        .iter()
        .any(|m| m.measurement_dates.corrected_decimal_age.is_none())
    {
        return Err(failed());
    }
    let gestation = EchoedGestation {
        weeks: request.gestation_weeks,
        days: request.gestation_days,
    };
    let mut out = Vec::with_capacity(measurements.len() * 4700 + 2);
    out.push(b'[');
    for (i, measurement) in measurements.iter().enumerate() {
        if i > 0 {
            out.push(b',');
        }
        let observation = ObservationFields {
            observation_date: measurement.measurement_dates.observation_date,
            observation_value: measurement.observation_value,
            bone_age: None,
            bone_age_type: None,
            bone_age_sds: None,
            bone_age_centile: None,
            bone_age_text: None,
            events_text: None,
        };
        out.extend_from_slice(&render_measurement(
            &state.store,
            measurement,
            &observation,
            gestation,
        ));
    }
    out.push(b']');
    Ok(out)
}

async fn utilities_mid_parental_height(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let value = match parse_body(&RequestBody::new(&headers, body)) {
        Ok(v) => v,
        Err(response) => return *response,
    };
    let request = match validate::mid_parental_height_request(&value) {
        Ok(r) => r,
        Err(errors) => return validation_failed(errors),
    };
    match mid_parental_height_response(&state, &request) {
        Ok(bytes) => raw_json(StatusCode::OK, bytes),
        Err(Rejection::Report(error)) => route_failed(*error),
        Err(Rejection::Respond(response)) => *response,
    }
}

/// The upstream `mid_parental_height_endpoint`.
fn mid_parental_height_response(
    state: &AppState,
    request: &validate::MidParentalHeightRequest,
) -> Result<Vec<u8>, Rejection> {
    let reference = match request.reference.as_str() {
        "cdc" => GrowthReference::Cdc,
        "who" => GrowthReference::Who,
        _ => GrowthReference::UkWho,
    };
    let sex: Sex = request.sex.parse().expect("validated literal");
    let method = MeasurementMethod::Height;
    let age = adult_age(reference);
    let unavailable = |error: &DataError| Rejection::Respond(Box::new(data_unavailable(error)));
    let raised = |error: &dyn std::fmt::Display| {
        Rejection::Report(Box::new(RouteError::request(format!("Error: {error}"))))
    };
    let mut tables = Vec::new();
    // The references the parents' SDS were calculated against.
    let mut calculated_against = Vec::new();

    let mut parent_sds = |height: f64, parent_sex: Sex| match sds_for_measurement(
        &state.store,
        reference,
        age,
        method,
        height,
        parent_sex,
    ) {
        Ok(result) => {
            tables.push(result.table);
            calculated_against.push(result.growth_reference);
            Ok(result.sds)
        }
        // At adult age L is 1, so upstream's SDS for a height of zero or
        // less is real and very negative: below the limit.
        Err(CalcError::InvalidObservation(v)) if v == f64::INFINITY => Ok(f64::INFINITY),
        Err(CalcError::InvalidObservation(_)) => Ok(f64::NEG_INFINITY),
        Err(CalcError::Data(error)) => Err(unavailable(&error)),
        Err(error) => Err(raised(&error)),
    };
    let maternal = parent_sds(request.height_maternal, Sex::Female)?;
    let paternal = parent_sds(request.height_paternal, Sex::Male)?;

    let parent_error = |parent: &str, side: &str, field: &'static str| {
        RouteError::value_error(
            format!(
                "Error: The {parent} height is {side} SD. Please check the accuracy of the {parent} height and try again."
            ),
            field,
        )
    };
    let mut errors = Vec::new();
    if paternal < -8.0 {
        errors.push(parent_error("paternal", "< -8", "height_paternal"));
    }
    if maternal < -8.0 {
        errors.push(parent_error("maternal", "< -8", "height_maternal"));
    }
    if paternal > 8.0 {
        errors.push(parent_error("paternal", "> 8", "height_paternal"));
    }
    if maternal > 8.0 {
        errors.push(parent_error("maternal", "> 8", "height_maternal"));
    }
    if !errors.is_empty() {
        return Err(Rejection::Respond(Box::new(json_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            &Detail { detail: errors },
        ))));
    }

    let sds = mid_parental_sds(maternal, paternal);
    // `round(centile(z), 3)`, then `round(..., 2)` in the response, both on
    // SciPy's NumPy float.
    let centile = numpy_round(numpy_round(normal::cdf(sds) * 100.0, 3), 2);
    let (lower, upper) = expected_range(sds);

    let mut line = |z: f64| -> Result<CentileData, Rejection> {
        let values = [python_round(z, 3)];
        let spec = CentileSpec::Custom {
            values: &values,
            is_sds: true,
        };
        match chart(&state.store, reference, method, sex, spec) {
            Ok(chart) => {
                tables.extend_from_slice(&chart.provenance.tables_read);
                Ok(CentileData::new(&chart, method, sex))
            }
            Err(ChartError::Data(error)) => Err(unavailable(&error)),
            // Upstream prints the exception and returns an empty line.
            Err(_) => Ok(CentileData::default()),
        }
    };
    let centile_data = line(sds)?;
    let lower_centile_data = line(lower)?;
    let upper_centile_data = line(upper)?;

    let mut height_at = |z: f64| -> Result<f64, Rejection> {
        match measurement_from_sds(
            &state.store,
            reference,
            PyFloat::Builtin(z),
            method,
            sex,
            age,
            false,
        ) {
            Ok((Some(height), table)) => {
                tables.push(table);
                Ok(height.round(1).value())
            }
            Ok((None, _)) => Err(raised(&"no height at this SDS")),
            Err(CalcError::Data(error)) => Err(unavailable(&error)),
            Err(error) => Err(raised(&error)),
        }
    };
    let upper_height = height_at(upper)?;
    let lower_height = height_at(lower)?;
    let height = height_at(sds)?;

    // Both parents' SDS are within 8 SD by now, so both were calculated.
    let Some(&growth_reference) = calculated_against.first() else {
        return Err(raised(&"no parental SDS was calculated"));
    };
    let response = MidParentalHeightResponse {
        provenance: record(&state.store, &Provenance::new(growth_reference, tables)),
        mid_parental_height: height,
        mid_parental_height_sds: python_round(sds, 3),
        mid_parental_height_centile: centile,
        mid_parental_height_centile_data: centile_data,
        mid_parental_height_lower_centile_data: lower_centile_data,
        mid_parental_height_upper_centile_data: upper_centile_data,
        mid_parental_height_upper_value: Some(upper_height),
        mid_parental_height_lower_value: Some(lower_height),
    };
    Ok(pyjson::to_vec(&response))
}

/// Python's `str()` of a list of floats, for messages that embed one.
fn python_list(values: &[f64]) -> String {
    let items: Vec<String> = values
        .iter()
        .map(|&v| crate::pyfmt::python_repr(v))
        .collect();
    format!("[{}]", items.join(", "))
}

async fn chart_coordinates(
    state: Arc<AppState>,
    body: RequestBody,
    reference: GrowthReference,
) -> Response {
    let value = match parse_body(&body) {
        Ok(v) => v,
        Err(response) => return *response,
    };
    let request = match validate::chart_coordinate_request(&value) {
        Ok(r) => r,
        Err(errors) => return validation_failed(errors),
    };
    let api = references::definition(reference).api;
    if let Some(only) = api.only
        && (request.sex != only.sex.as_str() || request.measurement_method != only.method.as_str())
    {
        return route_failed(RouteError::request(only.chart));
    }
    let sex: Sex = request.sex.parse().expect("validated literal");
    let method: MeasurementMethod = request
        .measurement_method
        .parse()
        .expect("validated literal");
    let render = |spec: CentileSpec<'_>| -> Result<Vec<u8>, ChartError> {
        let chart = chart(&state.store, reference, method, sex, spec)?;
        let provenance = record(&state.store, &chart.provenance);
        Ok(pyjson::to_vec(&ChartResponse::new(
            &chart, method, sex, provenance,
        )))
    };
    let not_found = |name: &str| {
        route_failed(RouteError::request(format!(
            "Item not found: chart-data/{name}-{}-{sex}-{method}.json",
            reference.as_str()
        )))
    };

    let result = match &request.centile_format {
        CentileFormatRequest::Custom(values) => match render(CentileSpec::Custom {
            values,
            is_sds: request.is_sds,
        }) {
            Ok(bytes) => Ok(Arc::new(bytes)),
            Err(ChartError::NonFinite(_)) => {
                return route_failed(RouteError::request(format!(
                    "Error creating {sex} {method} {} chart on the server, using {} centile format.",
                    api.chart_name,
                    python_list(values)
                )));
            }
            Err(error) => Err(error),
        },
        CentileFormatRequest::Named(name) => match CentileFormat::from_name(name) {
            None => return not_found(name),
            Some(format) => {
                let key = (reference, format, method, sex);
                let cached = state
                    .chart_cache
                    .lock()
                    .expect("cache lock")
                    .get(&key)
                    .cloned();
                match cached {
                    Some(bytes) => Ok(bytes),
                    None => render(CentileSpec::Named(format)).map(|bytes| {
                        let bytes = Arc::new(bytes);
                        state
                            .chart_cache
                            .lock()
                            .expect("cache lock")
                            .insert(key, bytes.clone());
                        bytes
                    }),
                }
            }
        },
        CentileFormatRequest::Null => return not_found("None"),
    };
    match result {
        Ok(bytes) => raw_json(StatusCode::OK, bytes.as_ref().clone()),
        Err(ChartError::Data(error)) => data_unavailable(&error),
        // Unreachable: every named collection's centiles lie strictly between
        // 0 and 100, so each has a finite SDS.
        Err(error @ ChartError::NonFinite(_)) => {
            detail(StatusCode::INTERNAL_SERVER_ERROR, &error.to_string())
        }
    }
}

impl IntoResponse for Rejection {
    fn into_response(self) -> Response {
        match self {
            Self::Report(error) => route_failed(*error),
            Self::Respond(response) => *response,
        }
    }
}

/// Whether the UK-WHO tables are all present, for a warning at start-up.
#[must_use]
pub fn uk_who_ready(store: &ReferenceStore) -> bool {
    use crate::TableId;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_content_type_is_json_as_python_parses_it() {
        let is_json = |value: &[u8]| {
            labelled_json(Some(
                &HeaderValue::from_bytes(value).expect("a header value"),
            ))
        };
        assert!(labelled_json(None));
        for json in [
            &b""[..],
            b"application/json",
            b"APPLICATION/JSON; charset=utf-8",
            b"application/json \t; q=1",
            b"application/json\xa0",
            b"application/problem+json",
        ] {
            assert!(is_json(json), "{}", String::from_utf8_lossy(json));
        }
        for other in [
            &b" "[..],
            b"text/plain",
            b"application/json/",
            b"application /json",
            b"application/jsonp",
            b"\xe1pplication/json",
        ] {
            assert!(!is_json(other), "{}", String::from_utf8_lossy(other));
        }
    }

    #[cfg(feature = "embedded-data")]
    #[test]
    fn a_measurement_is_written_with_the_provenance_the_engine_recorded() {
        let store = ReferenceStore::embedded().expect("embedded tables");
        let input = MeasurementInput {
            reference: GrowthReference::UkWho,
            birth_date: jiff::civil::date(2020, 4, 12),
            observation_date: jiff::civil::date(2028, 6, 12),
            gestation: Gestation::new(40, 0).expect("term"),
            sex: Sex::Female,
            method: MeasurementMethod::Height,
            observation_value: 115.0,
        };
        let mut measurement = measure(&store, &input).expect("a measurement");
        // A server that named the reference it routed to, rather than the
        // one the engine used, would still write uk-who here.
        measurement.provenance.growth_reference = GrowthReference::Who;
        let observation = ObservationFields {
            observation_date: input.observation_date,
            observation_value: input.observation_value,
            bone_age: None,
            bone_age_type: None,
            bone_age_sds: None,
            bone_age_centile: None,
            bone_age_text: None,
            events_text: None,
        };
        let gestation = EchoedGestation {
            weeks: Some(40),
            days: Some(0),
        };
        let body = render_measurement(&store, &measurement, &observation, gestation);
        let text = String::from_utf8(body).expect("UTF-8");
        assert!(
            text.starts_with(r#"{"provenance":{"growth_reference":"who","#),
            "{}",
            &text[..120]
        );
    }

    #[test]
    fn secrets_compare_only_when_equal() {
        assert!(same_secret(b"Bearer key", b"Bearer key"));
        assert!(!same_secret(b"Bearer kez", b"Bearer key"));
        assert!(!same_secret(b"Bearer ke", b"Bearer key"));
        assert!(!same_secret(b"", b"Bearer key"));
        assert!(same_secret(b"", b""));
    }
}
