// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

//! `opengrowth data`: reference-table status and import.

use std::path::PathBuf;
use std::process::ExitCode;

use opengrowth_data::{FileState, ImportAction, MANIFEST, Terms};
use serde::Serialize;

use super::Context;
use crate::paths::{self, DataDir};

/// `data` subcommands.
#[derive(Debug, clap::Subcommand)]
pub enum DataCommand {
    /// Show which reference tables are present and verified.
    Status,
    /// Verify reference tables in DIR and copy them into opengrowth's data directory.
    ///
    /// DIR uses the rcpchgrowth `data_tables` layout; an installed rcpchgrowth
    /// package's `data_tables` directory works as-is. Every file must match its
    /// published checksum; one bad file aborts the import before anything is written.
    Import {
        /// Directory to import from.
        #[arg(value_parser = paths::tilde_path, value_hint = clap::ValueHint::DirPath)]
        source: PathBuf,
    },
    /// List every table opengrowth knows, with its origin and terms.
    Manifest,
}

/// Runs a `data` subcommand.
pub fn run(context: &Context, command: &DataCommand) -> anyhow::Result<ExitCode> {
    match command {
        DataCommand::Status => status(context),
        DataCommand::Import { source } => import(context, source),
        DataCommand::Manifest => manifest(context),
    }
}

#[derive(Serialize)]
struct StatusReport {
    directory: DataDir,
    tables: Vec<TableStatus>,
}

#[derive(Serialize)]
struct TableStatus {
    table: &'static str,
    file: &'static str,
    state: &'static str,
    terms: &'static str,
    expected_sha256: &'static str,
    found_sha256: Option<String>,
    detail: Option<String>,
}

fn status(context: &Context) -> anyhow::Result<ExitCode> {
    let directory = paths::reference_dir(context.data_dir.as_deref())?;
    let tables: Vec<TableStatus> = opengrowth_data::inspect(&directory.path)
        .into_iter()
        .map(|status| {
            let (state, found_sha256, detail) = match status.state {
                FileState::Missing => ("missing", None, None),
                FileState::Verified => ("verified", Some(status.source.sha256.to_owned()), None),
                FileState::ChecksumMismatch { actual } => ("checksum mismatch", Some(actual), None),
                FileState::Unreadable { message } => ("unreadable", None, Some(message)),
            };
            TableStatus {
                table: status.source.id.as_str(),
                file: status.source.file,
                state,
                terms: status.source.terms.label(),
                expected_sha256: status.source.sha256,
                found_sha256,
                detail,
            }
        })
        .collect();
    let all_usable = tables
        .iter()
        .all(|t| matches!(t.state, "verified" | "missing"));
    let report = StatusReport { directory, tables };

    if !context.format.print_structured(&report)? {
        let dir = &report.directory;
        println!(
            "reference data: {} ({}{})",
            dir.path.display(),
            dir.source,
            if dir.exists {
                ""
            } else {
                "; does not exist yet"
            }
        );
        println!();
        println!("{:<16} {:<18} {:<17} FILE", "TABLE", "STATE", "TERMS");
        for t in &report.tables {
            println!("{:<16} {:<18} {:<17} {}", t.table, t.state, t.terms, t.file);
        }
    }
    if report.tables.iter().any(|t| t.state == "missing") {
        eprintln!("hint: import missing tables with `opengrowth data import <DIR>`");
    }
    Ok(if all_usable {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    })
}

#[derive(Serialize)]
struct ImportReport {
    from: PathBuf,
    to: PathBuf,
    tables: Vec<ImportedTable>,
}

#[derive(Serialize)]
struct ImportedTable {
    table: &'static str,
    file: &'static str,
    action: &'static str,
}

fn import(context: &Context, source: &std::path::Path) -> anyhow::Result<ExitCode> {
    anyhow::ensure!(source.is_dir(), "{} is not a directory", source.display());
    let destination = paths::import_dir()?;
    let outcomes = opengrowth_data::import(source, &destination.path)?;
    let report = ImportReport {
        from: source.to_owned(),
        to: destination.path,
        tables: outcomes
            .iter()
            .map(|o| ImportedTable {
                table: o.source.id.as_str(),
                file: o.source.file,
                action: match o.action {
                    ImportAction::Imported => "imported",
                    ImportAction::AlreadyPresent => "already present",
                    ImportAction::NotInSource => "not in source",
                },
            })
            .collect(),
    };
    if !context.format.print_structured(&report)? {
        println!("importing into {}", report.to.display());
        for t in &report.tables {
            println!("  {:<16} {}", t.table, t.action);
        }
    }
    eprintln!(
        "note: reference tables carry their own terms, separate from opengrowth's licence. \
         UK90 is owned by UKRI; make sure your use is permitted (`opengrowth data manifest`)."
    );
    Ok(ExitCode::SUCCESS)
}

#[derive(Serialize)]
struct ManifestEntry {
    table: &'static str,
    title: &'static str,
    file: &'static str,
    sha256: &'static str,
    origin: &'static str,
    terms: &'static str,
    owner: Option<&'static str>,
    note: &'static str,
}

fn manifest(context: &Context) -> anyhow::Result<ExitCode> {
    let entries: Vec<ManifestEntry> = MANIFEST
        .iter()
        .map(|s| {
            let (owner, note) = match s.terms {
                Terms::Restricted { owner, note } => (Some(owner), note),
                Terms::Unrecorded { note } => (None, note),
            };
            ManifestEntry {
                table: s.id.as_str(),
                title: s.title,
                file: s.file,
                sha256: s.sha256,
                origin: s.origin,
                terms: s.terms.label(),
                owner,
                note,
            }
        })
        .collect();
    if !context.format.print_structured(&entries)? {
        for e in &entries {
            println!("{}  {}", e.table, e.title);
            println!("    file    {}", e.file);
            println!("    sha256  {}", e.sha256);
            println!("    origin  {}", e.origin);
            match e.owner {
                Some(owner) => println!("    terms   {}: {} ({})", e.terms, owner, e.note),
                None => println!("    terms   {} ({})", e.terms, e.note),
            }
        }
    }
    Ok(ExitCode::SUCCESS)
}
