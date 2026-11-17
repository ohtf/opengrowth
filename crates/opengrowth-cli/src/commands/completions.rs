// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

//! `opengrowth completions <SHELL>`.

use std::process::ExitCode;

use clap::CommandFactory;
use clap_complete::Shell;

/// Arguments for `completions`.
#[derive(Debug, clap::Args)]
pub struct Args {
    /// Shell to generate completions for.
    pub shell: Shell,
}

/// Writes the completion script to stdout.
pub fn run(args: &Args) -> anyhow::Result<ExitCode> {
    let mut command = crate::Cli::command();
    clap_complete::generate(
        args.shell,
        &mut command,
        "opengrowth",
        &mut std::io::stdout(),
    );
    Ok(ExitCode::SUCCESS)
}
