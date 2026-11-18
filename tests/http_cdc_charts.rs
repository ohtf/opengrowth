// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

#![cfg(all(feature = "server", feature = "embedded-data"))]

//! Profile B for `POST /cdc/chart-coordinates`: every captured request
//! replayed and compared byte for byte outside `provenance`, with the
//! named-format files regenerated as for UK-WHO.

mod common;

#[tokio::test]
async fn cdc_chart_coordinates_match_fastapi_byte_for_byte() {
    common::http::check_chart_golden("expected/http-cdc-charts.json").await;
}
