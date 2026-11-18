// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

#![cfg(all(feature = "server", feature = "embedded-data"))]

//! Profile B for `POST /cdc/fictional-child-data`, with every random draw
//! fixed as in the goldens.

mod common;

/// Category 3 fixes (`spec/standard.md` section 6.3), each checked to still
/// differ in exactly the stated way.
const KNOWN_DIFFERENCES: &[(&str, &str)] = &[(
    "sds 16 bmi",
    "CDC BMI at a centile that rounds to 100 is infinite upstream, and the response then fails with a 500",
)];

const ROUTE_ERROR: &str = r#"{"detail":[{"loc":["request"],"msg":"Not possible to create CDC fictional child data.","type":"value_error","input":null}]}"#;

#[tokio::test]
async fn cdc_fictional_child_data_matches_fastapi_byte_for_byte() {
    common::http::check_fictional_child_golden(
        "expected/http-cdc-fictional-child.json",
        KNOWN_DIFFERENCES,
        ROUTE_ERROR,
    )
    .await;
}
