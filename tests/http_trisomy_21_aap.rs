// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

#![cfg(all(feature = "server", feature = "embedded-data"))]

//! Profile B for AAP Trisomy 21: replays every calculation and bulk-calculation
//! request captured from the FastAPI server and compares the body byte for
//! byte outside `provenance`.

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
async fn trisomy_21_aap_http_matches_fastapi_byte_for_byte() {
    common::http::check_reference_golden("expected/http-trisomy-21-aap.json", KNOWN_DIFFERENCES)
        .await;
}
