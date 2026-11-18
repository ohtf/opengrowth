// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

#![cfg(all(feature = "server", feature = "embedded-data"))]

//! Profile B for `POST /who/fictional-child-data`, with every random draw
//! fixed as in the goldens. WHO has no data below term, so a series starting
//! before birth never produces the measurement dated before birth that makes
//! UK-WHO fail upstream: there are no known differences.

mod common;

#[tokio::test]
async fn who_fictional_child_data_matches_fastapi_byte_for_byte() {
    common::http::check_fictional_child_golden("expected/http-who-fictional-child.json", &[], "")
        .await;
}
