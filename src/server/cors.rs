// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Cross-origin resource sharing, so browser applications on other origins,
//! such as the demo client, can call the API (`spec/queries.md` Q8).
//!
//! Only browsers enforce CORS. Other clients send no `Origin` header, and
//! nothing here changes what they receive.
//!
//! By default any origin may call the API, without credentials. This is what
//! the upstream server's own middleware (Starlette's `CORSMiddleware`) sends
//! when configured with `allow_credentials=False` and placed in front of
//! authorisation, and `tests/http_cors.rs` checks it byte for byte. Upstream
//! allows credentials as well, which the CORS protocol does not permit
//! together with "any origin", and the API authenticates with a header, not
//! cookies, so it has no use for credentialed requests. Preflights are
//! answered before authorisation because browsers never send credentials on
//! them; the request that follows is still checked.

use std::sync::Arc;

use axum::body::Body;
use axum::extract::{Request, State};
use axum::http::{HeaderMap, HeaderValue, Method, StatusCode, header};
use axum::middleware::Next;
use axum::response::Response;

use super::AppState;

/// Which origins may call the API from a browser.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum CorsPolicy {
    /// Any origin (`Access-Control-Allow-Origin: *`). The default.
    #[default]
    AnyOrigin,
    /// Only these origins, each echoed back when it calls.
    Origins(Vec<HeaderValue>),
    /// No CORS headers at all, for a deployment whose gateway adds its own.
    Off,
}

impl CorsPolicy {
    fn allows(&self, origin: &HeaderValue) -> bool {
        match self {
            Self::AnyOrigin => true,
            Self::Origins(origins) => origins.contains(origin),
            Self::Off => false,
        }
    }
}

/// Starlette's `ALL_METHODS`, which upstream allows.
const ALLOWED_METHODS: &str = "DELETE, GET, HEAD, OPTIONS, PATCH, POST, PUT";
/// How long a browser may cache a preflight, in seconds; Starlette's default.
const MAX_AGE: &str = "600";

/// The CORS middleware. It must be the outermost layer, so that preflights
/// are answered before authorisation and every response gets its headers.
pub async fn cors(State(state): State<Arc<AppState>>, request: Request, next: Next) -> Response {
    let policy = &state.cors;
    let Some(origin) = request.headers().get(header::ORIGIN).cloned() else {
        return next.run(request).await;
    };
    if *policy == CorsPolicy::Off {
        return next.run(request).await;
    }
    let requested_method = request
        .headers()
        .get(header::ACCESS_CONTROL_REQUEST_METHOD)
        .cloned();
    if request.method() == Method::OPTIONS
        && let Some(requested_method) = requested_method
    {
        return preflight(policy, &origin, &requested_method, request.headers());
    }
    let has_cookie = request.headers().contains_key(header::COOKIE);
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    match policy {
        // Starlette echoes the origin, rather than "*", when a cookie was sent.
        CorsPolicy::AnyOrigin if has_cookie => echo_origin(headers, origin),
        CorsPolicy::AnyOrigin => {
            headers.insert(
                header::ACCESS_CONTROL_ALLOW_ORIGIN,
                HeaderValue::from_static("*"),
            );
        }
        CorsPolicy::Origins(_) if policy.allows(&origin) => echo_origin(headers, origin),
        CorsPolicy::Origins(_) | CorsPolicy::Off => {}
    }
    response
}

/// Answers a preflight, as Starlette's `preflight_response` does.
fn preflight(
    policy: &CorsPolicy,
    origin: &HeaderValue,
    requested_method: &HeaderValue,
    request_headers: &HeaderMap,
) -> Response {
    let mut failures = Vec::new();
    let mut builder = Response::builder();
    if *policy == CorsPolicy::AnyOrigin {
        builder = builder.header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*");
    } else {
        builder = builder.header(header::VARY, "Origin");
        if policy.allows(origin) {
            builder = builder.header(header::ACCESS_CONTROL_ALLOW_ORIGIN, origin.clone());
        } else {
            failures.push("origin");
        }
    }
    builder = builder
        .header(header::ACCESS_CONTROL_ALLOW_METHODS, ALLOWED_METHODS)
        .header(header::ACCESS_CONTROL_MAX_AGE, MAX_AGE);
    let method_allowed = requested_method
        .to_str()
        .is_ok_and(|m| ALLOWED_METHODS.split(", ").any(|allowed| allowed == m));
    if !method_allowed {
        failures.push("method");
    }
    // Every request header is allowed, so the requested ones are echoed.
    if let Some(requested) = request_headers.get(header::ACCESS_CONTROL_REQUEST_HEADERS) {
        builder = builder.header(header::ACCESS_CONTROL_ALLOW_HEADERS, requested.clone());
    }
    let (status, text) = if failures.is_empty() {
        (StatusCode::OK, "OK".to_owned())
    } else {
        (
            StatusCode::BAD_REQUEST,
            format!("Disallowed CORS {}", failures.join(", ")),
        )
    };
    builder
        .status(status)
        .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .body(Body::from(text))
        .expect("static headers are valid")
}

/// Sets the allowed origin to the caller's and adds `Origin` to `Vary`.
fn echo_origin(headers: &mut HeaderMap, origin: HeaderValue) {
    headers.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, origin);
    let vary = match headers.get(header::VARY).and_then(|v| v.to_str().ok()) {
        Some(existing) if !existing.is_empty() => format!("{existing}, Origin"),
        _ => "Origin".to_owned(),
    };
    headers.insert(
        header::VARY,
        HeaderValue::from_str(&vary).expect("from valid header values"),
    );
}

/// Parses `--cors-allow-origin` values: `*` alone for any origin, or exact
/// origins such as `https://app.example.org` (scheme, host, and any port,
/// with no path), which is what browsers send.
///
/// # Errors
///
/// For an empty list, `*` mixed with origins, or a value that is not an origin.
pub fn parse_origins(values: &[String]) -> Result<CorsPolicy, String> {
    if values.is_empty() {
        return Err("no origins given".to_owned());
    }
    if values.iter().any(|v| v == "*") {
        return if values.len() == 1 {
            Ok(CorsPolicy::AnyOrigin)
        } else {
            Err("`*` allows every origin, so it cannot be combined with others".to_owned())
        };
    }
    let origins = values
        .iter()
        .map(|value| {
            let rest = value
                .strip_prefix("https://")
                .or_else(|| value.strip_prefix("http://"))
                .ok_or_else(|| format!("{value:?} is not an origin: it must start with https:// or http://"))?;
            if rest.is_empty() || rest.contains('/') {
                return Err(format!(
                    "{value:?} is not an origin: give the scheme, host, and any port, with no path or trailing slash"
                ));
            }
            HeaderValue::from_str(value).map_err(|_| format!("{value:?} is not a valid header value"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(CorsPolicy::Origins(origins))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn origins_are_parsed_strictly() {
        assert_eq!(parse_origins(&["*".into()]), Ok(CorsPolicy::AnyOrigin));
        assert_eq!(
            parse_origins(&["https://a.example".into(), "http://localhost:3000".into()]),
            Ok(CorsPolicy::Origins(vec![
                HeaderValue::from_static("https://a.example"),
                HeaderValue::from_static("http://localhost:3000"),
            ]))
        );
        for bad in [
            "https://a.example/",
            "a.example",
            "https://",
            "https://a.example/app",
        ] {
            assert!(parse_origins(&[bad.into()]).is_err(), "{bad}");
        }
        assert!(parse_origins(&["*".into(), "https://a.example".into()]).is_err());
        assert!(parse_origins(&[]).is_err());
    }
}
