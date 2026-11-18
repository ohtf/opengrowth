// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

#![cfg(all(feature = "server", feature = "embedded-data"))]

//! Profile B for `POST /trisomy-21/chart-coordinates`, with the named-format
//! files regenerated as for UK-WHO. Needs the reference's imported tables.

mod common;

use opengrowth::TableId;

/// Category 3 fixes: the Trisomy 21 builder rounds an infinite SDS without
/// failing, so the response holds an infinity and fails to serialise.
const KNOWN_DIFFERENCES: &[(&str, &str)] = &[
    (
        "error centile 0",
        "an infinite SDS reaches the response, which fails with a 500",
    ),
    (
        "error centile 100",
        "an infinite SDS reaches the response, which fails with a 500",
    ),
    (
        "error centile 150",
        "an infinite SDS reaches the response, which fails with a 500",
    ),
];

#[tokio::test]
async fn trisomy_21_chart_coordinates_match_fastapi_byte_for_byte() {
    if !common::tables_available(&[TableId::Trisomy21]) {
        return;
    }
    common::http::check_chart_golden(
        "expected/http-trisomy-21-charts.json",
        KNOWN_DIFFERENCES,
        false,
    )
    .await;
}
