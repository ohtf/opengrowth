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

use std::sync::Arc;

use common::http::{Case, assert_no_failures, problems, replay};
use common::{DATA_DIR_ENV, Golden, read_json};
use opengrowth::server::{AppState, router};
use opengrowth::store::ReferenceStore;

/// Intentional differences from the pinned server (`spec/standard.md`
/// section 6.3), each checked to still hold.
const KNOWN_DIFFERENCES: &[(&str, &str)] = &[(
    "every named-format chart",
    "upstream serves chart files last regenerated on 2025-01-11, which differ from its own engine; \
     opengrowth calculates them with rcpchgrowth 4.6.5",
)];

#[derive(serde::Deserialize)]
struct ChartCase {
    #[serde(flatten)]
    case: Case,
    stale_masked_sha256: Option<String>,
}

#[tokio::test]
async fn uk_who_chart_coordinates_match_fastapi_byte_for_byte() {
    let dir = std::env::var_os(DATA_DIR_ENV).map(std::path::PathBuf::from);
    let app = router(Arc::new(AppState::new(
        ReferenceStore::standard(dir.as_deref()).expect("tables load"),
        None,
    )));
    let golden: Golden<Vec<ChartCase>> = read_json("expected/http-uk-who-charts.json");
    let mut failures = Vec::new();
    let mut stale_differs = 0;
    let mut named = 0;
    for chart_case in &golden.cases {
        let case = &chart_case.case;
        let outcome = replay(app.clone(), case).await;
        let found = problems(case, &outcome);
        if !found.is_empty() {
            failures.push(format!("{}: {}", case.name, found.join("; ")));
        }
        if let Some(stale) = &chart_case.stale_masked_sha256 {
            named += 1;
            if *stale != case.masked_sha256 {
                stale_differs += 1;
            }
        }
    }
    eprintln!(
        "{} cases; {} differ. Upstream's stored chart files differ from rcpchgrowth 4.6.5 for {stale_differs} of {named} named formats.",
        golden.cases.len(),
        failures.len()
    );
    let (name, reason) = KNOWN_DIFFERENCES[0];
    if stale_differs != named {
        failures.push(format!(
            "{name} is listed as a known difference ({reason}), but only {stale_differs} of {named} differ"
        ));
    }
    assert_no_failures(&failures, golden.cases.len());
}
