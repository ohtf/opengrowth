// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-FileCopyrightText: Royal College of Paediatrics and Child Health
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Calculation provenance (`spec/provenance.md`), ported case for case from
//! upstream's two suites: the engine's (`rcpchgrowth/tests/test_provenance.py`
//! in `rcpchgrowth-python`) and the server's (`tests/test_provenance.py` in
//! the pinned `digital-growth-charts-server`), with opengrowth's own engine
//! and server names (section 3). The server suite's last case, that the
//! OpenAPI document declares provenance, waits for that document (`R15`).
//!
//! Beyond the ported cases, provenance must survive every way a measurement
//! can fail to produce values (section 6), name only the tables read
//! (invariant 8), and come with charts and mid-parental height too.

use std::collections::BTreeSet;

use jiff::civil::{Date, date};
use opengrowth::{GrowthReference, Sex};

/// One case for each reference, as upstream's engine suite chooses them:
/// inside each reference's ages, and a girl for Turner syndrome. The dates
/// are what upstream's `observation_date_for_age` gives for 5, 10, and 1
/// years after its `BIRTH_DATE`, 1 January 2015.
const REFERENCE_CASES: [(GrowthReference, Sex, Date); 6] = [
    (GrowthReference::UkWho, Sex::Female, date(2020, 1, 1)),
    (GrowthReference::Trisomy21, Sex::Female, date(2020, 1, 1)),
    (GrowthReference::Trisomy21Aap, Sex::Female, date(2020, 1, 1)),
    (
        GrowthReference::TurnersSyndrome,
        Sex::Female,
        date(2024, 12, 31),
    ),
    (GrowthReference::Cdc, Sex::Male, date(2024, 12, 31)),
    (GrowthReference::Who, Sex::Male, date(2016, 1, 1)),
];

/// Each reference's route prefix and the `growth_reference` its responses
/// carry, as upstream's server suite lists them.
const ENDPOINTS: [(&str, &str); 6] = [
    ("uk-who", "uk-who"),
    ("trisomy-21", "trisomy-21"),
    ("trisomy-21-aap", "trisomy-21-aap"),
    ("turner", "turners-syndrome"),
    ("cdc", "cdc"),
    ("who", "who"),
];

/// The drift guard (invariant 7): a seventh reference fails here until it
/// has provenance cases of its own, rather than going untested.
#[test]
fn every_reference_has_provenance_cases() {
    let all: BTreeSet<GrowthReference> = GrowthReference::ALL.into_iter().collect();
    let cases: BTreeSet<GrowthReference> = REFERENCE_CASES.iter().map(|case| case.0).collect();
    assert_eq!(
        cases, all,
        "every reference needs an engine provenance case"
    );
    assert_eq!(
        REFERENCE_CASES.len(),
        all.len(),
        "one case for each reference"
    );

    let defined: BTreeSet<GrowthReference> = opengrowth::references::ALL
        .iter()
        .map(|definition| definition.reference)
        .collect();
    assert_eq!(defined, all);

    let routes: BTreeSet<(&str, &str)> = opengrowth::references::ALL
        .iter()
        .map(|definition| (definition.api.prefix, definition.reference.as_str()))
        .collect();
    let endpoints: BTreeSet<(&str, &str)> = ENDPOINTS.into_iter().collect();
    assert_eq!(
        endpoints, routes,
        "every route prefix needs a server provenance case"
    );
}

/// Upstream's schema refuses any reference outside the six, and the
/// shortened `turner`; opengrowth's type cannot hold one.
#[test]
fn growth_reference_has_only_the_canonical_values() {
    for bad in [
        "invalid",
        "turner",
        "Turner's syndrome",
        "UK-WHO",
        "down-syndrome",
    ] {
        assert!(
            serde_json::from_value::<GrowthReference>(serde_json::json!(bad)).is_err(),
            "{bad}"
        );
        assert!(bad.parse::<GrowthReference>().is_err(), "{bad}");
    }
}

#[cfg(feature = "embedded-data")]
mod engine {
    use super::*;
    use opengrowth::advice::CentileFormat;
    use opengrowth::chart::{CentileSpec, chart};
    use opengrowth::fictional_child::{FictionalChildParams, IntervalUnit, fictional_child};
    use opengrowth::provenance::{ENGINE, Record, is_commit};
    use opengrowth::store::ReferenceStore;
    use opengrowth::{
        Gestation, Measurement, MeasurementInput, MeasurementMethod, TableId, measure,
    };

    /// Upstream's `BIRTH_DATE`.
    const BIRTH_DATE: Date = date(2015, 1, 1);

    fn store() -> ReferenceStore {
        ReferenceStore::embedded().expect("embedded tables")
    }

    fn term() -> Gestation {
        Gestation::new(40, 0).expect("term")
    }

    fn measurement(
        reference: GrowthReference,
        sex: Sex,
        observation_date: Date,
        method: MeasurementMethod,
        observation_value: f64,
        gestation: Gestation,
    ) -> Measurement {
        let input = MeasurementInput {
            reference,
            birth_date: BIRTH_DATE,
            observation_date,
            gestation,
            sex,
            method,
            observation_value,
        };
        measure(&store(), &input).expect("a measurement result")
    }

    fn height(
        reference: GrowthReference,
        sex: Sex,
        observation_date: Date,
        value: f64,
    ) -> Measurement {
        measurement(
            reference,
            sex,
            observation_date,
            MeasurementMethod::Height,
            value,
            term(),
        )
    }

    #[test]
    fn growth_reference_is_the_reference_calculated_against() {
        for (reference, sex, observation_date) in REFERENCE_CASES {
            let measurement = height(reference, sex, observation_date, 110.0);
            assert_eq!(measurement.provenance.growth_reference, reference);
        }
    }

    #[test]
    fn the_calculation_engine_is_always_present() {
        for (reference, sex, observation_date) in REFERENCE_CASES {
            let engine = height(reference, sex, observation_date, 110.0)
                .provenance
                .calculation_engine;
            assert_eq!(engine, ENGINE, "{reference}");
            assert_eq!(engine.name, "opengrowth", "{reference}");
            assert_eq!(engine.version, env!("CARGO_PKG_VERSION"), "{reference}");
            assert!(!engine.version.is_empty(), "{reference}");
            assert!(
                !engine.commit.is_empty() && is_commit(engine.commit),
                "{reference}"
            );
        }
    }

    #[test]
    fn trisomy_21_and_trisomy_21_aap_are_distinguishable() {
        let at_five = date(2020, 1, 1);
        let uk = height(GrowthReference::Trisomy21, Sex::Female, at_five, 100.0);
        let aap = height(GrowthReference::Trisomy21Aap, Sex::Female, at_five, 100.0);
        assert_eq!(uk.provenance.growth_reference, GrowthReference::Trisomy21);
        assert_eq!(
            aap.provenance.growth_reference,
            GrowthReference::Trisomy21Aap
        );
        assert_ne!(
            uk.provenance.growth_reference,
            aap.provenance.growth_reference
        );
    }

    #[test]
    fn turners_syndrome_keeps_its_canonical_spelling() {
        let store = store();
        let measurement = height(
            GrowthReference::TurnersSyndrome,
            Sex::Female,
            date(2024, 12, 31),
            120.0,
        );
        let record = Record::new(&measurement.provenance, |id| store.table_used(id), None);
        let text = serde_json::to_string(&record).expect("serialisable");
        assert!(
            text.starts_with(r#"{"growth_reference":"turners-syndrome","#),
            "{text}"
        );
    }

    #[test]
    fn provenance_is_present_when_the_numerical_result_is_not() {
        let before_birth = date(2014, 12, 31);
        let measurement = height(GrowthReference::UkWho, Sex::Female, before_birth, 50.0);
        assert_eq!(measurement.values.corrected_sds, None);
        assert_eq!(
            measurement.provenance.growth_reference,
            GrowthReference::UkWho
        );
        // Nothing could be calculated, so no table was read (invariant 8).
        assert!(measurement.provenance.tables_read.is_empty());
    }

    #[test]
    fn provenance_survives_every_way_a_measurement_can_lack_values() {
        use GrowthReference::{Cdc, TurnersSyndrome, UkWho};
        use MeasurementMethod::{Height, Weight};
        let preterm = Gestation::new(30, 0).expect("30 weeks");
        // What is missing; the measurement; and whether it can read a table.
        let cases = [
            (
                "no reference data at this age",
                TurnersSyndrome,
                Sex::Female,
                date(2015, 7, 2),
                Height,
                65.0,
                term(),
                false,
            ),
            (
                "a measurement the reference does not cover",
                TurnersSyndrome,
                Sex::Female,
                date(2024, 12, 31),
                Weight,
                30.0,
                term(),
                false,
            ),
            (
                "a preterm baby before term, for CDC",
                Cdc,
                Sex::Male,
                date(2015, 2, 1),
                Weight,
                1.9,
                preterm,
                true,
            ),
            (
                "an implausible measurement",
                UkWho,
                Sex::Female,
                date(2020, 1, 1),
                Height,
                300.0,
                term(),
                true,
            ),
        ];
        for (case, reference, sex, observed, method, value, gestation, reads_tables) in cases {
            let measurement = measurement(reference, sex, observed, method, value, gestation);
            let lacks_values = measurement.values.corrected_sds.is_none()
                || measurement.observation_value_error.is_some();
            assert!(
                lacks_values,
                "{case}: expected no corrected SDS, or an implausible value"
            );
            assert_eq!(measurement.provenance.growth_reference, reference, "{case}");
            assert_eq!(measurement.provenance.calculation_engine, ENGINE, "{case}");
            if !reads_tables {
                assert!(measurement.provenance.tables_read.is_empty(), "{case}");
            }
        }
    }

    #[test]
    fn a_measurement_names_each_table_it_read_once() {
        // Born at 28 weeks and measured 8 weeks later: the corrected age is
        // in UK90's preterm table and the chronological age in WHO 2006's.
        let gestation = Gestation::new(28, 0).expect("28 weeks");
        let measurement = measurement(
            GrowthReference::UkWho,
            Sex::Female,
            date(2015, 2, 26),
            MeasurementMethod::Height,
            47.0,
            gestation,
        );
        assert_eq!(
            measurement.provenance.tables_read,
            [TableId::Uk90Preterm, TableId::Who2006Infant]
        );
    }

    #[test]
    fn fictional_child_measurements_carry_provenance() {
        let ages = [5.0, 5.0, 5.0, 10.0, 10.0, 1.0];
        for ((reference, sex, _), age) in REFERENCE_CASES.into_iter().zip(ages) {
            let params = FictionalChildParams {
                reference,
                method: MeasurementMethod::Height,
                sex,
                start_age: f64::max(age - 1.0, 0.0),
                end_age: age + 1.0,
                gestation: term(),
                interval_unit: IntervalUnit::Months,
                interval_number: 6,
                start_sds: 0.0,
                drift_range: None,
                noise_range: None,
            };
            let measurements = fictional_child(&store(), &params, &mut || 0.5).expect("a series");
            assert!(!measurements.is_empty(), "{reference}");
            for measurement in measurements {
                assert_eq!(measurement.provenance.growth_reference, reference);
                assert_eq!(measurement.provenance.calculation_engine.name, "opengrowth");
            }
        }
    }

    #[test]
    fn a_chart_carries_the_provenance_of_the_reference_that_drew_it() {
        let store = store();
        for definition in opengrowth::references::ALL {
            let drawn = chart(
                &store,
                definition.reference,
                MeasurementMethod::Height,
                Sex::Female,
                CentileSpec::Named(CentileFormat::ColeNineCentiles),
            )
            .expect("a chart");
            assert_eq!(drawn.provenance.growth_reference, definition.reference);
            assert_eq!(drawn.provenance.calculation_engine, ENGINE);
            assert!(
                !drawn.provenance.tables_read.is_empty(),
                "{}",
                definition.reference
            );
        }
    }
}

#[cfg(all(feature = "server", feature = "embedded-data"))]
mod server {
    use std::sync::Arc;

    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use opengrowth::TableId;
    use opengrowth::provenance::{COMMIT, is_commit};
    use opengrowth::server::response::API_SERVER;
    use opengrowth::server::{AppState, router};
    use opengrowth::store::{ReferenceStore, manifest};
    use serde_json::{Value, json};
    use tower::ServiceExt;

    use super::ENDPOINTS;

    /// Upstream's `CALCULATION_REQUEST`.
    const CALCULATION_REQUEST: &str = r#"{"birth_date": "2020-04-12", "observation_date": "2028-06-12", "observation_value": 115, "sex": "female", "gestation_weeks": 40, "gestation_days": 0, "measurement_method": "height"}"#;

    async fn post(path: &str, body: &str) -> (StatusCode, String) {
        let store = ReferenceStore::embedded().expect("embedded tables");
        let app = router(Arc::new(AppState::new(store, None)));
        let request = Request::builder()
            .method("POST")
            .uri(path)
            .header("host", "testserver")
            .header("content-type", "application/json")
            .body(Body::from(body.to_owned()))
            .expect("request");
        let response = app.oneshot(request).await.expect("response");
        let status = response.status();
        let bytes = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        (status, String::from_utf8(bytes.to_vec()).expect("UTF-8"))
    }

    fn engine() -> Value {
        json!({ "name": "opengrowth", "version": env!("CARGO_PKG_VERSION"), "commit": COMMIT })
    }

    fn api_server() -> Value {
        json!({ "name": "opengrowth-server", "version": env!("CARGO_PKG_VERSION"), "commit": COMMIT })
    }

    /// The provenance at the start of `body`, after checking that it has the
    /// contract's four keys, in order, and nothing else.
    fn provenance(body: &str) -> Value {
        let parsed: Value = serde_json::from_str(body).expect("JSON");
        let provenance = parsed["provenance"].clone();
        let keys: Vec<&str> = provenance
            .as_object()
            .expect("an object")
            .keys()
            .map(String::as_str)
            .collect();
        let mut expected = [
            "growth_reference",
            "calculation_engine",
            "api_server",
            "reference_data",
        ];
        let at: Vec<usize> = expected
            .iter()
            .map(|key| body.find(&format!("\"{key}\"")).expect(key))
            .collect();
        assert!(
            at.windows(2).all(|w| w[0] < w[1]),
            "key order: {}",
            &body[..200]
        );
        assert!(
            body.starts_with(r#"{"provenance":{"growth_reference":"#),
            "{}",
            &body[..80]
        );
        expected.sort_unstable();
        let mut keys = keys;
        keys.sort_unstable();
        assert_eq!(keys, expected);
        provenance
    }

    #[tokio::test]
    async fn calculation_passes_through_the_engine_provenance() {
        for (prefix, growth_reference) in ENDPOINTS {
            let (status, body) = post(&format!("/{prefix}/calculation"), CALCULATION_REQUEST).await;
            assert_eq!(status, StatusCode::OK, "{prefix}: {body}");
            let provenance = provenance(&body);
            assert_eq!(provenance["growth_reference"], growth_reference);
            assert_eq!(provenance["calculation_engine"], engine(), "{prefix}");
            assert_eq!(provenance["api_server"], api_server(), "{prefix}");
        }
    }

    #[tokio::test]
    async fn bulk_calculation_passes_through_provenance_without_changing_inline_errors() {
        let request = r#"{"birth_date": "2020-04-12", "sex": "female", "gestation_weeks": 40, "gestation_days": 0, "measurement_method": "height", "observations": [{"observation_date": "2028-06-12", "observation_value": 115}, {"observation_date": "2019-06-12", "observation_value": 115}]}"#;
        let (status, body) = post("/uk-who/bulk-calculation", request).await;
        assert_eq!(status, StatusCode::OK, "{body}");
        let results: Value = serde_json::from_str(&body).expect("JSON");
        let first = &results["results"][0]["provenance"];
        assert_eq!(first["growth_reference"], "uk-who");
        assert_eq!(first["api_server"], api_server());
        assert_eq!(results["results"][1]["type"], "value_error");
        assert!(results["results"][1].get("provenance").is_none());
    }

    #[tokio::test]
    async fn fictional_child_measurements_pass_through_provenance() {
        let request = r#"{"measurement_method": "height", "sex": "female", "start_chronological_age": 0, "end_age": 0.1, "gestation_weeks": 40, "gestation_days": 0, "measurement_interval_type": "days", "measurement_interval_number": 30, "start_sds": 0, "drift": false, "noise": false}"#;
        let (status, body) = post("/uk-who/fictional-child-data", request).await;
        assert_eq!(status, StatusCode::OK, "{body}");
        let measurements: Vec<Value> = serde_json::from_str(&body).expect("JSON");
        assert!(!measurements.is_empty());
        for measurement in measurements {
            assert_eq!(measurement["provenance"]["growth_reference"], "uk-who");
            assert_eq!(measurement["provenance"]["api_server"], api_server());
        }
    }

    #[test]
    fn the_server_reports_its_own_build_identity() {
        assert_eq!(API_SERVER.name, "opengrowth-server");
        assert_ne!(API_SERVER.name, "rcpch/digital-growth-charts-server");
        assert!(!API_SERVER.version.is_empty());
        assert!(is_commit(API_SERVER.commit));
    }

    #[tokio::test]
    async fn charts_and_mid_parental_height_pass_through_provenance() {
        let chart_request = r#"{"sex": "female", "measurement_method": "height", "centile_format": "cole-nine-centiles"}"#;
        for (prefix, growth_reference) in ENDPOINTS {
            let (status, body) = post(&format!("/{prefix}/chart-coordinates"), chart_request).await;
            assert_eq!(status, StatusCode::OK, "{prefix}");
            let provenance = provenance(&body);
            assert_eq!(provenance["growth_reference"], growth_reference);
            assert_eq!(provenance["api_server"], api_server(), "{prefix}");
        }
        for reference in ["uk-who", "cdc", "who"] {
            let request = format!(
                r#"{{"height_paternal": 180, "height_maternal": 165, "sex": "male", "reference": "{reference}"}}"#
            );
            let (status, body) = post("/utilities/mid-parental-height", &request).await;
            assert_eq!(status, StatusCode::OK, "{reference}: {body}");
            let provenance = provenance(&body);
            assert_eq!(provenance["growth_reference"], reference);
            assert_eq!(provenance["calculation_engine"], engine(), "{reference}");
        }
    }

    #[tokio::test]
    async fn reference_data_names_each_table_read_with_its_file() {
        // Born at 28 weeks and measured 8 weeks later: UK90 preterm at the
        // corrected age and WHO 2006 at the chronological age.
        let request = r#"{"birth_date": "2020-04-12", "observation_date": "2020-06-07", "observation_value": 47, "sex": "female", "gestation_weeks": 28, "gestation_days": 0, "measurement_method": "height"}"#;
        let (status, body) = post("/uk-who/calculation", request).await;
        assert_eq!(status, StatusCode::OK, "{body}");
        let expected: Vec<Value> = [TableId::Uk90Preterm, TableId::Who2006Infant]
            .into_iter()
            .map(|id| {
                let source = manifest::source(id);
                json!({ "table": id.as_str(), "file": source.file, "sha256": source.sha256, "source": "embedded" })
            })
            .collect();
        assert_eq!(provenance(&body)["reference_data"], json!(expected));
    }
}
