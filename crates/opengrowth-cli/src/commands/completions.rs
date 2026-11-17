// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

//! `opengrowth completions <SHELL>` (or `og completions <SHELL>`).

use std::process::ExitCode;

use clap::CommandFactory;
use clap_complete::Shell;

/// Arguments for `completions`.
#[derive(Debug, clap::Args)]
pub struct Args {
    /// Shell to generate completions for.
    pub shell: Shell,
}

/// Writes the completion script to stdout, for the name the binary was run
/// as (`opengrowth` or `og`), so each name completes.
pub fn run(args: &Args) -> anyhow::Result<ExitCode> {
    let mut command = crate::Cli::command();
    clap_complete::generate(
        args.shell,
        &mut command,
        invoked_name(),
        &mut std::io::stdout(),
    );
    Ok(ExitCode::SUCCESS)
}

fn invoked_name() -> &'static str {
    let invoked = std::env::args_os().next().and_then(|arg| {
        std::path::Path::new(&arg)
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
    });
    if invoked.as_deref() == Some("og") {
        "og"
    } else {
        "opengrowth"
    }
}
