// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

#![cfg(all(feature = "server", feature = "embedded-data"))]

//! Profile B for UK-WHO: replays every request captured from the FastAPI
//! server and compares status, recorded headers, and the body byte for byte
//! outside `provenance` (`spec/standard.md` section 4.2).
//!
//! Uses the embedded tables, or the files in `$OPENGROWTH_REFERENCE_DATA_DIR`
//! layered over them if it is set.

mod common;

/// Cases where opengrowth deliberately differs from upstream, each a
/// Category 3 fix listed in `spec/standard.md` section 6.3. Each must still
/// differ in exactly the stated way: upstream 500, opengrowth 422. A case
/// that starts to match, or differs differently, fails the test, so this
/// list cannot go stale or hide a regression.
const KNOWN_DIFFERENCES: &[(&str, &str)] = &[
    (
        "validation birth date not string",
        "upstream strptime raises TypeError, which escapes as a 500",
    ),
    (
        "validation birth date null",
        "upstream strptime raises TypeError, which escapes as a 500",
    ),
];

#[tokio::test]
async fn uk_who_http_matches_fastapi_byte_for_byte() {
    common::http::check_reference_golden("expected/http-uk-who.json", KNOWN_DIFFERENCES).await;
}
