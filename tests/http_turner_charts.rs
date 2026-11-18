// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

#![cfg(all(feature = "server", feature = "embedded-data"))]

//! Profile B for `POST /turner/chart-coordinates`, with the named-format
//! files regenerated as for UK-WHO. Needs the reference's imported tables.

mod common;

use opengrowth::TableId;

#[tokio::test]
async fn turner_chart_coordinates_match_fastapi_byte_for_byte() {
    if !common::tables_available(&[TableId::Turner]) {
        return;
    }
    common::http::check_chart_golden("expected/http-turner-charts.json", &[], false).await;
}
