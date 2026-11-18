// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

#![cfg(all(feature = "server", feature = "embedded-data"))]

//! Profile B for `POST /uk-who/chart-coordinates`: every captured request
//! replayed and compared byte for byte outside `provenance`.
//!
//! The goldens are the pinned server's responses with its named-format chart
//! files regenerated from rcpchgrowth 4.6.5, because the pinned server serves
//! files from January 2025 that no longer match its own engine (see
//! `conformance/generators/http_uk_who_charts.py`). That is the one known
//! difference, and the test checks that it still holds.

mod common;

#[tokio::test]
async fn uk_who_chart_coordinates_match_fastapi_byte_for_byte() {
    common::http::check_chart_golden("expected/http-uk-who-charts.json", &[], true).await;
}
