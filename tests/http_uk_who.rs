// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

#![cfg(all(feature = "server", feature = "embedded-data"))]

//! Profile B for UK-WHO: replays every request captured from the FastAPI
//! server and compares status, recorded headers, and the body byte for byte
//! outside `provenance` (`spec/standard.md` section 4.2).
//!
//! Uses the embedded tables, or the files in `$OPENGROWTH_REFERENCE_DATA_DIR`
//! layered over them if it is set.

mod common;

use std::sync::Arc;

use common::http::{Case, assert_no_failures, identical_including_provenance, problems, replay};
use common::{DATA_DIR_ENV, Golden, read_json};
use opengrowth::server::{AppState, router};
use opengrowth::store::ReferenceStore;

#[derive(serde::Deserialize)]
struct Cases {
    curated: Vec<Case>,
    r_vectors: Vec<Case>,
}

/// Cases where opengrowth deliberately differs from upstream, each a
/// Category 3 fix listed in `spec/standard.md` section 6.3. Each must still
/// differ in exactly the stated way: upstream 500, opengrowth 422. A case
/// that starts to match, or differs differently, fails the test, so this
/// list cannot go stale or hide a regression.
const KNOWN_DIFFERENCES: &[(&str, &str)] = &[
    (
        "validation birth date not string",
        "upstream strptime raises TypeError, which escapes as a 500",
    ),
    (
        "validation birth date null",
        "upstream strptime raises TypeError, which escapes as a 500",
    ),
];

#[tokio::test]
async fn uk_who_http_matches_fastapi_byte_for_byte() {
    let dir = std::env::var_os(DATA_DIR_ENV).map(std::path::PathBuf::from);
    let store = || ReferenceStore::standard(dir.as_deref()).expect("tables load");
    let open = router(Arc::new(AppState::new(store(), None)));
    let locked = router(Arc::new(AppState::new(
        store(),
        Some("test-key".to_owned()),
    )));

    let golden: Golden<Cases> = read_json("expected/http-uk-who.json");
    eprintln!("reference server environment: {}", golden.environment);
    let mut failures = Vec::new();
    let mut identical = 0;
    let mut masked_identical = 0;
    let all: Vec<&Case> = golden
        .cases
        .curated
        .iter()
        .chain(&golden.cases.r_vectors)
        .collect();
    for case in &all {
        let app = if case.name.starts_with("auth ") {
            locked.clone()
        } else {
            open.clone()
        };
        let outcome = replay(app, case).await;
        if KNOWN_DIFFERENCES.iter().any(|(name, _)| *name == case.name) {
            let as_expected = case.status == 500
                && outcome.status == 422
                && outcome.body.contains("\"value_error\"");
            if !as_expected {
                failures.push(format!(
                    "{}: listed as a known difference but behaved differently",
                    case.name
                ));
            }
            continue;
        }
        if identical_including_provenance(case, &outcome) {
            identical += 1;
        }
        let found = problems(case, &outcome);
        if found.is_empty() {
            masked_identical += 1;
        } else {
            failures.push(format!("{}: {}", case.name, found.join("; ")));
        }
    }
    eprintln!(
        "{} cases: {masked_identical} byte-identical outside provenance, {identical} identical including it, {} known differences",
        all.len(),
        KNOWN_DIFFERENCES.len()
    );
    assert_no_failures(&failures, all.len());
}
