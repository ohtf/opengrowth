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

/// Requests for references opengrowth has not ported yet, which must say so
/// with a 501 until they are. Remove each as its reference lands.
const PENDING: &[&str] = &["pending cdc"];

#[tokio::test]
async fn mid_parental_height_matches_fastapi_byte_for_byte() {
    let dir = std::env::var_os(DATA_DIR_ENV).map(std::path::PathBuf::from);
    let store = ReferenceStore::standard(dir.as_deref()).expect("tables load");
    let app = router(Arc::new(AppState::new(store, None)));
    let golden: Golden<Vec<Case>> = read_json("expected/http-utilities-mid-parental-height.json");
    let mut failures = Vec::new();
    let mut pending = 0;
    for case in &golden.cases {
        let outcome = replay(app.clone(), case).await;
        if PENDING.contains(&case.name.as_str()) {
            pending += 1;
            if outcome.status != 501 {
                failures.push(format!(
                    "{}: listed as pending but returned {}",
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
        "{} cases: {} byte-identical outside provenance, {pending} pending references",
        golden.cases.len(),
        golden.cases.len() - pending - failures.len()
    );
    assert_eq!(pending, PENDING.len(), "a pending case is missing");
    assert_no_failures(&failures, golden.cases.len());
}
