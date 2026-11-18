// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

#![cfg(all(feature = "server", feature = "embedded-data"))]

//! Profile B for `POST /trisomy-21-aap/fictional-child-data`, with every random
//! draw fixed as in the goldens. Needs the reference's imported tables.

mod common;

use opengrowth::TableId;

#[tokio::test]
async fn trisomy_21_aap_fictional_child_data_matches_fastapi_byte_for_byte() {
    if !common::tables_available(&[TableId::Trisomy21AapInfant, TableId::Trisomy21AapChild]) {
        return;
    }
    common::http::check_fictional_child_golden(
        "expected/http-trisomy-21-aap-fictional-child.json",
        &[],
        "",
    )
    .await;
}
