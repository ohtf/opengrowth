// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

//! One module per command family, each with a `run()`.

pub mod calculate;
pub mod completions;
pub mod data;
pub mod openapi;
pub mod serve;

use std::path::PathBuf;

use crate::cli::OutputFormat;

/// Global options every command can read.
#[derive(Debug)]
pub struct Context {
    /// Output format.
    pub format: OutputFormat,
    /// `--data-dir`, if given.
    pub data_dir: Option<PathBuf>,
}
