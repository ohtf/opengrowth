// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

//! `opengrowth openapi`: print the OpenAPI document that describes the HTTP
//! API, for a deployment to publish with its own address.

use std::process::ExitCode;

use crate::server::openapi::{self, Server};

#[derive(Debug, clap::Args)]
pub struct Args {
    /// A URL the API is served at, for the document's list of servers.
    /// Repeat it for more than one. Without it, the list names
    /// `opengrowth serve` on this computer.
    #[arg(long = "server", value_name = "URL")]
    pub servers: Vec<String>,
}

/// Prints the document as JSON, whatever `--format` says, since OpenAPI
/// documents are JSON.
pub fn run(args: &Args) -> anyhow::Result<ExitCode> {
    let servers = if args.servers.is_empty() {
        openapi::default_servers()
    } else {
        args.servers
            .iter()
            .map(|url| Server {
                url: url.clone(),
                description: None,
            })
            .collect()
    };
    println!(
        "{}",
        serde_json::to_string_pretty(&openapi::document(&servers))?
    );
    Ok(ExitCode::SUCCESS)
}
