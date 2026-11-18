// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

//! `opengrowth calculate`: SDS and centile for one measurement, at
//! chronological and corrected age.
//!
//! Sex and gestation are required rather than defaulted, because agents are
//! among this command's main users and a silently assumed gestation changes
//! the corrected result (SAFETY.md, "Agent misuse through the CLI").

use std::collections::BTreeMap;
use std::process::ExitCode;

use crate::dates::{chronological_decimal_age, corrected_decimal_age};
use crate::store::ReferenceStore;
use crate::{
    CalcError, DataError, ENGINE_NAME, ENGINE_VERSION, Gestation, GrowthReference,
    MeasurementMethod, Sex, TableId, centile, sds_for_measurement,
};
use jiff::civil::Date;
use serde::Serialize;

use super::Context;
use crate::cli::paths;

/// Arguments for `calculate`.
#[derive(Debug, clap::Args)]
pub struct Args {
    /// Growth reference.
    #[arg(long, default_value_t = GrowthReference::UkWho)]
    pub reference: GrowthReference,
    /// Sex of the child.
    #[arg(long)]
    pub sex: Sex,
    /// Date of birth, YYYY-MM-DD.
    #[arg(long, value_name = "DATE")]
    pub birth_date: Date,
    /// Date the measurement was taken, YYYY-MM-DD.
    #[arg(long, value_name = "DATE")]
    pub observation_date: Date,
    /// Gestation at birth as WEEKS+DAYS, for example 40+0 or 32+4.
    #[arg(long, value_name = "WEEKS+DAYS")]
    pub gestation: Gestation,
    /// What was measured.
    #[arg(long, value_name = "METHOD")]
    pub measurement: MeasurementMethod,
    /// The measurement: cm for height and ofc, kg for weight, kg/m² for bmi.
    #[arg(long)]
    pub value: f64,
}

#[derive(Debug, Serialize)]
struct Report {
    provenance: Provenance,
    input: Input,
    chronological: AgeResult,
    corrected: AgeResult,
}

#[derive(Debug, Serialize)]
struct Provenance {
    growth_reference: GrowthReference,
    calculation_engine: Engine,
    /// The tables actually read, never the tables configured.
    reference_data: Vec<TableUsed>,
}

#[derive(Debug, Serialize)]
struct Engine {
    name: &'static str,
    version: &'static str,
    commit: &'static str,
}

#[derive(Debug, Serialize)]
struct TableUsed {
    table: &'static str,
    file: &'static str,
    sha256: &'static str,
    source: &'static str,
}

#[derive(Debug, Serialize)]
struct Input {
    birth_date: String,
    observation_date: String,
    gestation: String,
    sex: Sex,
    measurement_method: MeasurementMethod,
    observation_value: f64,
}

#[derive(Debug, Serialize)]
struct AgeResult {
    decimal_age: Option<f64>,
    sds: Option<f64>,
    centile: Option<f64>,
    source_table: Option<&'static str>,
    error: Option<String>,
    #[serde(skip)]
    missing_table: bool,
}

impl AgeResult {
    fn failed(decimal_age: Option<f64>, error: String) -> Self {
        Self {
            decimal_age,
            sds: None,
            centile: None,
            source_table: None,
            error: Some(error),
            missing_table: false,
        }
    }
}

/// Runs `calculate`.
pub fn run(context: &Context, args: &Args) -> anyhow::Result<ExitCode> {
    let dir = paths::reference_dir(context.data_dir.as_deref())?;
    let store = ReferenceStore::standard(Some(&dir.path))?;

    let mut tables_read = Vec::new();
    let mut at_age = |age: Result<f64, String>| -> AgeResult {
        let age = match age {
            Ok(age) => age,
            Err(error) => return AgeResult::failed(None, error),
        };
        match sds_for_measurement(
            &store,
            args.reference,
            age,
            args.measurement,
            args.value,
            args.sex,
        ) {
            Ok(result) => {
                tables_read.push(result.table);
                AgeResult {
                    decimal_age: Some(age),
                    sds: Some(result.sds),
                    centile: Some(centile(result.sds)),
                    source_table: Some(result.table.as_str()),
                    error: None,
                    missing_table: false,
                }
            }
            Err(error) => AgeResult {
                missing_table: matches!(error, CalcError::Data(DataError::Unavailable { .. })),
                ..AgeResult::failed(Some(age), error.to_string())
            },
        }
    };

    let chronological = at_age(Ok(chronological_decimal_age(
        args.birth_date,
        args.observation_date,
    )));
    let corrected = at_age(
        corrected_decimal_age(args.birth_date, args.observation_date, args.gestation)
            .map_err(|e| e.to_string()),
    );

    let report = Report {
        provenance: Provenance {
            growth_reference: args.reference,
            calculation_engine: Engine {
                name: ENGINE_NAME,
                version: ENGINE_VERSION,
                commit: "unknown",
            },
            reference_data: tables_used(&store, tables_read),
        },
        input: Input {
            birth_date: args.birth_date.to_string(),
            observation_date: args.observation_date.to_string(),
            gestation: args.gestation.to_string(),
            sex: args.sex,
            measurement_method: args.measurement,
            observation_value: args.value,
        },
        chronological,
        corrected,
    };

    if !context.format.print_structured(&report)? {
        print_text(&report);
    }
    if report.chronological.missing_table || report.corrected.missing_table {
        eprintln!(
            "hint: reference tables are read from {} ({}). \
             Import them with `opengrowth data import <DIR>`, or check `opengrowth data status`.",
            dir.path.display(),
            dir.source
        );
    }
    let any_result = report.chronological.sds.is_some() || report.corrected.sds.is_some();
    Ok(if any_result {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    })
}

fn tables_used(store: &ReferenceStore, mut read: Vec<TableId>) -> Vec<TableUsed> {
    read.sort();
    read.dedup();
    read.into_iter()
        .filter_map(|id| store.file_for(id))
        .map(|file| TableUsed {
            table: file.source.id.as_str(),
            file: file.source.file,
            sha256: file.source.sha256,
            source: file.origin.label(),
        })
        .collect()
}

fn unit(method: MeasurementMethod) -> &'static str {
    match method {
        MeasurementMethod::Height | MeasurementMethod::Ofc => "cm",
        MeasurementMethod::Weight => "kg",
        MeasurementMethod::Bmi => "kg/m²",
    }
}

fn reference_label(reference: GrowthReference) -> &'static str {
    match reference {
        GrowthReference::UkWho => "UK-WHO",
        GrowthReference::TurnersSyndrome => "Turner syndrome",
        GrowthReference::Trisomy21 => "Trisomy 21 (UK)",
        GrowthReference::Trisomy21Aap => "Trisomy 21 (AAP)",
        GrowthReference::Cdc => "CDC",
        GrowthReference::Who => "WHO",
    }
}

fn print_text(report: &Report) {
    let input = &report.input;
    println!(
        "{}: {} {} {}, {}, born {} at {} weeks, measured {}",
        reference_label(report.provenance.growth_reference),
        input.measurement_method,
        input.observation_value,
        unit(input.measurement_method),
        input.sex,
        input.birth_date,
        input.gestation,
        input.observation_date,
    );
    println!();
    println!(
        "{:<15} {:>12} {:>10} {:>9}  table",
        "", "decimal age", "SDS", "centile"
    );
    for (label, row) in [
        ("chronological", &report.chronological),
        ("corrected", &report.corrected),
    ] {
        let age = row
            .decimal_age
            .map_or_else(|| "-".to_owned(), |a| format!("{a:.4}"));
        match (row.sds, row.centile, &row.error) {
            (Some(sds), Some(centile), _) => println!(
                "{label:<15} {age:>12} {sds:>10.4} {centile:>9.2}  {}",
                row.source_table.unwrap_or("-")
            ),
            (_, _, error) => println!(
                "{label:<15} {age:>12} {:>10} {:>9}  {}",
                "-",
                "-",
                error.as_deref().unwrap_or("not calculated")
            ),
        }
    }
    println!();
    let mut checksums = BTreeMap::new();
    for table in &report.provenance.reference_data {
        checksums.insert(table.table, &table.sha256[..12]);
    }
    let tables: Vec<String> = checksums
        .iter()
        .map(|(t, s)| format!("{t} (sha256 {s}…)"))
        .collect();
    println!(
        "{} {}; tables: {}",
        report.provenance.calculation_engine.name,
        report.provenance.calculation_engine.version,
        if tables.is_empty() {
            "none".to_owned()
        } else {
            tables.join(", ")
        }
    );
    println!("Not for clinical use.");
}
