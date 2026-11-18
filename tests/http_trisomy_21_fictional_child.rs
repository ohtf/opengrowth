// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

#![cfg(all(feature = "server", feature = "embedded-data"))]

//! Profile B for `POST /trisomy-21/fictional-child-data`, with every random
//! draw fixed as in the goldens.

mod common;

#[tokio::test]
async fn trisomy_21_fictional_child_data_matches_fastapi_byte_for_byte() {
    common::http::check_fictional_child_golden(
        "expected/http-trisomy-21-fictional-child.json",
        &[],
        "",
    )
    .await;
}
