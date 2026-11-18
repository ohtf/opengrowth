// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

#![cfg(all(feature = "server", feature = "embedded-data"))]

//! Profile B for `POST /uk-who/fictional-child-data`: every captured request
//! replayed and compared byte for byte outside `provenance`.
//!
//! Upstream adds noise with `random.uniform`, so the goldens were captured
//! with every random draw fixed at [`FIXED_DRAW`], and the server here is
//! given the same draw (`AppState::with_fixed_noise`).

mod common;

use std::sync::Arc;

use common::http::{Case, assert_no_failures, problems, replay};
use common::{DATA_DIR_ENV, Golden, read_json};
use opengrowth::server::{AppState, router};
use opengrowth::store::ReferenceStore;

/// The draw `conformance/generators/http_uk_who_fictional_child.py` fixes.
const FIXED_DRAW: f64 = 0.8125;

/// Intentional differences from the pinned server (`spec/standard.md`
/// section 6.3), each checked to still hold.
const KNOWN_DIFFERENCES: &[(&str, &str)] = &[(
    "start before birth",
    "upstream's response model rejects a measurement dated before birth, which escapes as a 500",
)];

const ROUTE_ERROR: &str = r#"{"detail":[{"loc":["request"],"msg":"Not possible to create UK-WHO fictional child data.","type":"value_error","input":null}]}"#;

#[tokio::test]
async fn uk_who_fictional_child_data_matches_fastapi_byte_for_byte() {
    let dir = std::env::var_os(DATA_DIR_ENV).map(std::path::PathBuf::from);
    let store = ReferenceStore::standard(dir.as_deref()).expect("tables load");
    let app = router(Arc::new(
        AppState::new(store, None).with_fixed_noise(FIXED_DRAW),
    ));
    let golden: Golden<Vec<Case>> = read_json("expected/http-uk-who-fictional-child.json");
    let mut failures = Vec::new();
    let mut known = 0;
    for case in &golden.cases {
        let outcome = replay(app.clone(), case).await;
        if let Some((_, reason)) = KNOWN_DIFFERENCES
            .iter()
            .find(|(name, _)| *name == case.name)
        {
            known += 1;
            if !(case.status == 500 && outcome.status == 422 && outcome.body == ROUTE_ERROR) {
                failures.push(format!(
                    "{}: listed as a known difference ({reason}) but behaved differently: {} {}",
                    case.name, outcome.status, outcome.body
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

/// With real randomness, each noisy value lies within its noise bound of the
/// noise-free value (plus the rounding to one decimal place), and the noise
/// is not constant.
#[tokio::test]
async fn random_noise_stays_within_its_range() {
    let store = || ReferenceStore::standard(None).expect("tables load");
    let values = |body: &str| -> Vec<f64> {
        let list: Vec<serde_json::Value> = serde_json::from_str(body).expect("JSON list");
        list.iter()
            .map(|m| {
                m["child_observation_value"]["observation_value"]
                    .as_f64()
                    .unwrap()
            })
            .collect()
    };
    let request = |noise: bool| Case {
        name: "noise".to_owned(),
        method: "POST".to_owned(),
        path: "/uk-who/fictional-child-data".to_owned(),
        request_headers: std::collections::BTreeMap::new(),
        request_body: format!(
            r#"{{"measurement_method": "weight", "sex": "female", "noise": {noise}, "noise_range": 0.05}}"#
        ),
        status: 200,
        headers: std::collections::BTreeMap::new(),
        body_sha256: String::new(),
        masked_sha256: String::new(),
        body: None,
    };
    let app = || router(Arc::new(AppState::new(store(), None)));
    let exact = values(&replay(app(), &request(false)).await.body);
    let noisy = values(&replay(app(), &request(true)).await.body);
    assert_eq!(exact.len(), noisy.len());
    for (e, n) in exact.iter().zip(&noisy) {
        assert!(
            (n - e).abs() <= e * 0.05 + 0.1,
            "{n} is not within 5% of {e}"
        );
    }
    assert!(
        exact.iter().zip(&noisy).any(|(e, n)| e != n),
        "no noise was added"
    );
}
