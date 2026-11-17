// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The `opengrowth` command-line tool.
//!
//! Conventions follow the house-style `rust-cli.md`: clap derive, data on
//! stdout and hints on stderr, `--format text|json`, one module per command
//! family with a `run()` function, and path arguments that expand `~`.

#![forbid(unsafe_code)]

mod commands;
mod output;
mod paths;

use std::ffi::OsString;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};

pub use output::OutputFormat;

/// Growth calculations for children: SDS and centiles against UK-WHO and
/// other growth references.
///
/// Not for clinical use. See SAFETY.md.
#[derive(Debug, Parser)]
#[command(name = "opengrowth", version, arg_required_else_help = true)]
pub struct Cli {
    /// Output format.
    #[arg(long, global = true, value_enum, default_value_t = OutputFormat::Text)]
    pub format: OutputFormat,

    /// Directory of reference tables to use, instead of the imported ones.
    #[arg(
        long,
        global = true,
        env = paths::DATA_DIR_ENV,
        value_parser = paths::tilde_path,
        value_hint = clap::ValueHint::DirPath
    )]
    pub data_dir: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Command,
}

/// Top-level commands.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Calculate SDS and centiles for one measurement.
    Calculate(commands::calculate::Args),
    /// Inspect and import reference tables.
    #[command(subcommand)]
    Data(commands::data::DataCommand),
    /// Print a shell completion script.
    Completions(commands::completions::Args),
}

/// Parses arguments and runs the command.
///
/// # Errors
///
/// Any failure the command reports; `main` prints it and exits non-zero.
pub fn run<I, T>(args: I) -> anyhow::Result<ExitCode>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let cli = Cli::parse_from(args);
    let context = commands::Context {
        format: cli.format,
        data_dir: cli.data_dir,
    };
    match cli.command {
        Command::Calculate(args) => commands::calculate::run(&context, &args),
        Command::Data(command) => commands::data::run(&context, &command),
        Command::Completions(args) => commands::completions::run(&args),
    }
}
