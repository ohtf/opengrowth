// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Output formats.

use serde::Serialize;

/// How results are written to stdout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum OutputFormat {
    /// Aligned, human-readable text.
    Text,
    /// Pretty-printed JSON, one document per invocation.
    Json,
}

impl OutputFormat {
    /// Prints `value` as JSON and returns `true`, or returns `false` so the
    /// caller renders text.
    pub fn print_structured<T: Serialize>(self, value: &T) -> anyhow::Result<bool> {
        match self {
            Self::Text => Ok(false),
            Self::Json => {
                println!("{}", serde_json::to_string_pretty(value)?);
                Ok(true)
            }
        }
    }
}
