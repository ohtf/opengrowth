// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

#![cfg(all(feature = "server", feature = "embedded-data"))]

//! Profile B for `POST /trisomy-21-aap/chart-coordinates`, with the named-format
//! files regenerated as for UK-WHO. Needs the reference's imported tables.

mod common;

use opengrowth::TableId;

#[tokio::test]
async fn trisomy_21_aap_chart_coordinates_match_fastapi_byte_for_byte() {
    if !common::tables_available(&[TableId::Trisomy21AapInfant, TableId::Trisomy21AapChild]) {
        return;
    }
    common::http::check_chart_golden("expected/http-trisomy-21-aap-charts.json", &[], false).await;
}
