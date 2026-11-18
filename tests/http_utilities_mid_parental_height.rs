// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

#![cfg(all(feature = "server", feature = "embedded-data"))]

//! Profile B for `POST /utilities/mid-parental-height`: every captured
//! request replayed and compared byte for byte outside `provenance`.

mod common;

use std::sync::Arc;

use common::http::{Case, assert_no_failures, problems, replay};
use common::{DATA_DIR_ENV, Golden, read_json};
use opengrowth::server::{AppState, router};
use opengrowth::store::ReferenceStore;

/// Category 3 fixes (`spec/standard.md` section 6.3), each checked to still
/// be upstream's 500 and opengrowth's 422.
const KNOWN_DIFFERENCES: &[(&str, &str)] = &[(
    "cdc error negative heights",
    "CDC height L is not 1 at adult age, so a negative height gives upstream a complex SDS, and comparing it raises a TypeError, which escapes as a 500",
)];

#[tokio::test]
async fn mid_parental_height_matches_fastapi_byte_for_byte() {
    let dir = std::env::var_os(DATA_DIR_ENV).map(std::path::PathBuf::from);
    let store = ReferenceStore::standard(dir.as_deref()).expect("tables load");
    let app = router(Arc::new(AppState::new(store, None)));
    let golden: Golden<Vec<Case>> = read_json("expected/http-utilities-mid-parental-height.json");
    let mut failures = Vec::new();
    let mut known = 0;
    for case in &golden.cases {
        let outcome = replay(app.clone(), case).await;
        if let Some((_, reason)) = KNOWN_DIFFERENCES
            .iter()
            .find(|(name, _)| *name == case.name)
        {
            known += 1;
            if !(case.status == 500 && outcome.status == 422) {
                failures.push(format!(
                    "{}: listed as a known difference ({reason}) but returned {}",
                    case.name, outcome.status
                ));
            }
            continue;
        }
        let found = problems(case, &outcome);
        if !found.is_empty() {
            failures.push(format!("{}: {}", case.name, found.join("; ")));
        }
    }
    eprintln!(
        "{} cases: {} byte-identical outside provenance, {known} known differences",
        golden.cases.len(),
        golden.cases.len() - known - failures.len()
    );
    assert_eq!(
        known,
        KNOWN_DIFFERENCES.len(),
        "a known difference has no case"
    );
    assert_no_failures(&failures, golden.cases.len());
}
