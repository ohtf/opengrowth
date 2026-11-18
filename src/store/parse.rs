// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Parser for the `rcpchgrowth` `data_tables` JSON format.
//!
//! ```text
//! { "measurement": {
//!     "acknowledgement_text": "...",            (ignored)
//!     "height": { "male": [ { "decimal_age": 4.0, "L": 1, "M": 102.49, "S": 0.04008,
//!                             "interval": "months", "value": 48 } ... ],
//!                 "female": [ ... ] },
//!     "weight": ..., "bmi": ..., "ofc": ... } }
//! ```
//!
//! Only the four measurement keys are read. Row fields other than
//! `decimal_age`, `L`, `M`, `S`, and `sigma` are ignored; the checksum
//! already pins the exact content, so this parser does not need to be a
//! format validator, only exact about what it reads.

use std::collections::BTreeMap;

use crate::{LmsPoint, LmsTable, MeasurementMethod, Sex, TableId};
use serde::Deserialize;

#[derive(Deserialize)]
struct RawFile {
    measurement: BTreeMap<String, serde_json::Value>,
}

#[derive(Deserialize)]
struct RawPoint {
    decimal_age: f64,
    #[serde(rename = "L")]
    l: f64,
    #[serde(rename = "M")]
    m: f64,
    #[serde(rename = "S")]
    s: f64,
    #[serde(default)]
    sigma: Option<f64>,
}

/// Identifies one table within a store.
pub(crate) type TableKey = (TableId, MeasurementMethod, Sex);
/// A parsed table with its key.
pub(crate) type Keyed = (TableKey, LmsTable);

pub(crate) fn tables(id: TableId, bytes: &[u8]) -> Result<Vec<Keyed>, String> {
    let file: RawFile = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for method in MeasurementMethod::ALL {
        let Some(by_sex) = file.measurement.get(method.as_str()) else {
            continue;
        };
        for sex in [Sex::Male, Sex::Female] {
            let Some(rows) = by_sex.get(sex.as_str()) else {
                continue;
            };
            if is_placeholder(rows) {
                continue;
            }
            let rows: Vec<RawPoint> =
                serde_json::from_value(rows.clone()).map_err(|e| format!("{method} {sex}: {e}"))?;
            let points = rows
                .into_iter()
                .map(|r| LmsPoint {
                    age: r.decimal_age,
                    l: r.l,
                    m: r.m,
                    s: r.s,
                    sigma: r.sigma,
                })
                .collect();
            let table = LmsTable::new(points).map_err(|e| format!("{method} {sex}: {e}"))?;
            out.push(((id, method, sex), table));
        }
    }
    if out.is_empty() {
        return Err("no height, weight, bmi, or ofc tables found".to_owned());
    }
    Ok(out)
}

/// True if every row has empty-string L, M, and S: a placeholder, not data.
///
/// `uk90_preterm.json` carries one for BMI, which UK-WHO never reads because
/// it has no BMI data below 42 weeks. Skipping it makes the store report BMI
/// as missing from that table. A table with only some empty values is not a
/// placeholder and fails to parse.
fn is_placeholder(rows: &serde_json::Value) -> bool {
    let empty =
        |row: &serde_json::Value, key: &str| row.get(key).and_then(|v| v.as_str()) == Some("");
    rows.as_array().is_some_and(|rows| {
        !rows.is_empty()
            && rows
                .iter()
                .all(|row| ["L", "M", "S"].iter().all(|k| empty(row, k)))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_the_upstream_shape_and_ignores_extra_fields() {
        let json = br#"{"measurement": {
            "acknowledgement_text": "credit",
            "height": {"male": [
                {"decimal_age": 4.0, "L": 1, "M": 102.49, "S": 0.04008, "interval": "months", "value": 48},
                {"decimal_age": 4.083, "L": 1, "M": 103.1, "S": 0.0401, "interval": "months", "value": 49}
            ]}
        }}"#;
        let tables = tables(TableId::Uk90Child, json).unwrap();
        assert_eq!(tables.len(), 1);
        let ((_, method, sex), table) = &tables[0];
        assert_eq!((*method, *sex), (MeasurementMethod::Height, Sex::Male));
        assert_eq!(table.points()[0].m, 102.49);
    }

    #[test]
    fn rejects_rows_that_break_table_invariants() {
        let json = br#"{"measurement": {"weight": {"female": [
            {"decimal_age": 1.0, "L": 1, "M": 10, "S": 0.1},
            {"decimal_age": 0.5, "L": 1, "M": 9, "S": 0.1}
        ]}}}"#;
        assert!(
            tables(TableId::Uk90Child, json)
                .unwrap_err()
                .contains("ascending")
        );
    }

    #[test]
    fn all_placeholder_tables_are_skipped_but_partial_ones_are_rejected() {
        let placeholder = br#"{"measurement": {
            "bmi": {"male": [{"decimal_age": -0.3, "L": "", "M": "", "S": ""}]},
            "weight": {"male": [{"decimal_age": -0.3, "L": 1, "M": 0.6, "S": 0.15}]}
        }}"#;
        let tables_found = tables(TableId::Uk90Preterm, placeholder).unwrap();
        assert_eq!(tables_found.len(), 1);
        assert_eq!(tables_found[0].0.1, MeasurementMethod::Weight);

        let partial = br#"{"measurement": {"bmi": {"male": [
            {"decimal_age": -0.3, "L": "", "M": "", "S": ""},
            {"decimal_age": -0.2, "L": 1, "M": 0.6, "S": 0.15}
        ]}}}"#;
        assert!(tables(TableId::Uk90Preterm, partial).is_err());
    }
}
