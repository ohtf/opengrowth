// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

#![cfg(all(feature = "server", feature = "embedded-data"))]

//! Profile B for WHO: replays every calculation and bulk-calculation request
//! captured from the FastAPI server, the UK-WHO cases and WHO's own, and
//! compares the body byte for byte outside `provenance`.

mod common;

/// Category 3 fixes (`spec/standard.md` section 6.3), each checked to still
/// differ in exactly the stated way.
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
async fn who_http_matches_fastapi_byte_for_byte() {
    common::http::check_reference_golden("expected/http-who.json", KNOWN_DIFFERENCES).await;
}
