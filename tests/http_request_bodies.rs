// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

#![cfg(all(feature = "server", feature = "embedded-data"))]

//! How the server reads a request body (`spec/queries.md` Q16): which
//! `Content-Type` values it reads as JSON, and how it decodes the bytes.
//! Replays `conformance/expected/http-request-bodies.json` with exactly the
//! headers each case records, and compares as the other HTTP suites do.

mod common;

use std::sync::Arc;

use common::http::{Case, assert_no_failures, problems, replay_exact};
use opengrowth::server::{AppState, router};
use opengrowth::store::ReferenceStore;

/// Category 3 fixes (`spec/standard.md` section 6.3). Upstream refuses a
/// body labelled as something other than JSON with a 422 that shows the
/// body as text, but when the body is not UTF-8 its error handler cannot
/// decode it, and the 422 escapes as a 500. opengrowth returns the 422, with
/// the body decoded as UTF-8 with replacement characters. Each must still
/// differ in exactly that way, so the list cannot go stale.
const KNOWN_DIFFERENCES: [&str; 6] = [
    "calculation, UTF-16 with a byte-order mark, Content-Type 'text/plain'",
    "calculation, UTF-16LE without a byte-order mark, Content-Type 'text/plain'",
    "calculation, UTF-16BE without a byte-order mark, Content-Type 'text/plain'",
    "calculation, UTF-32 with a byte-order mark, Content-Type 'text/plain'",
    "calculation, UTF-32LE without a byte-order mark, Content-Type 'text/plain'",
    "calculation, Latin-1, which is not UTF-8, Content-Type 'text/plain'",
];

#[tokio::test]
async fn request_bodies_are_read_as_fastapi_reads_them() {
    let golden: common::Golden<Vec<Case>> = common::read_json("expected/http-request-bodies.json");
    let listed = golden
        .cases
        .iter()
        .filter(|case| KNOWN_DIFFERENCES.contains(&case.name.as_str()))
        .count();
    assert_eq!(
        listed,
        KNOWN_DIFFERENCES.len(),
        "every known difference names a case"
    );

    let store = ReferenceStore::embedded().expect("embedded tables");
    let app = router(Arc::new(AppState::new(store, None)));
    let mut failures = Vec::new();
    for case in &golden.cases {
        let outcome = replay_exact(app.clone(), case).await;
        if KNOWN_DIFFERENCES.contains(&case.name.as_str()) {
            let as_expected = case.status == 500
                && outcome.status == 422
                && outcome.body.starts_with(
                    r#"{"detail":[{"type":"model_attributes_type","loc":["body"],"msg":"Input should be a valid dictionary or object to extract fields from","input":""#,
                );
            if !as_expected {
                failures.push(format!(
                    "{}: listed as a known difference but behaved differently: {} {}",
                    case.name,
                    outcome.status,
                    &outcome.body[..outcome.body.len().min(120)]
                ));
            }
            continue;
        }
        let found = problems(case, &outcome);
        if !found.is_empty() {
            failures.push(format!("{}: {}", case.name, found.join("; ")));
        }
    }
    assert_no_failures(&failures, golden.cases.len());
}
