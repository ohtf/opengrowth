// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

//! `opengrowth serve`: the HTTP API.

use std::net::SocketAddr;
use std::process::ExitCode;
use std::sync::Arc;

use crate::server::AppState;
use crate::server::cors::{self, CorsPolicy};
use crate::store::ReferenceStore;
use anyhow::Context as _;

use super::Context;
use crate::cli::paths;

/// Arguments for `serve`.
#[derive(Debug, clap::Args)]
pub struct Args {
    /// Address to listen on.
    #[arg(long, env = "OPENGROWTH_BIND", default_value = "127.0.0.1:8000")]
    pub bind: SocketAddr,

    /// Require `Authorization: Bearer <KEY>` on every request except `/`.
    /// Read from the same variable as the upstream server.
    #[arg(long, env = "AUTHORIZATION_KEY", hide_env_values = true)]
    pub authorization_key: Option<String>,

    /// Number of worker threads (default: one per CPU).
    #[arg(long)]
    pub workers: Option<usize>,

    /// Origins browsers may call the API from: `*` for any, or exact origins
    /// such as `https://app.example.org`. Repeat the option, or separate them
    /// with commas.
    #[arg(
        long = "cors-allow-origin",
        value_name = "ORIGIN",
        env = "OPENGROWTH_CORS_ALLOW_ORIGINS",
        value_delimiter = ',',
        default_value = "*",
        conflicts_with = "no_cors"
    )]
    pub cors_allow_origins: Vec<String>,

    /// Send no CORS headers, for a deployment whose gateway adds its own.
    #[arg(long)]
    pub no_cors: bool,
}

/// Runs `serve` until interrupted.
pub fn run(context: &Context, args: &Args) -> anyhow::Result<ExitCode> {
    let dir = paths::reference_dir(context.data_dir.as_deref())?;
    let store = ReferenceStore::standard(Some(&dir.path))?;
    if !crate::server::uk_who_ready(&store) {
        eprintln!(
            "warning: UK-WHO tables are not all present in {} ({}); calculations will return 503. \
             Import them with `opengrowth data import <DIR>`.",
            dir.path.display(),
            dir.source
        );
    }
    let cors = if args.no_cors {
        CorsPolicy::Off
    } else {
        cors::parse_origins(&args.cors_allow_origins)
            .map_err(|e| anyhow::anyhow!("--cors-allow-origin: {e}"))?
    };
    let state = Arc::new(
        AppState::new(
            store,
            args.authorization_key.clone().filter(|k| !k.is_empty()),
        )
        .with_cors(cors),
    );

    let mut runtime = tokio::runtime::Builder::new_multi_thread();
    runtime.enable_all();
    if let Some(workers) = args.workers {
        runtime.worker_threads(workers);
    }
    let runtime = runtime.build().context("cannot start the async runtime")?;
    eprintln!(
        "opengrowth {} listening on http://{}{}",
        crate::ENGINE_VERSION,
        args.bind,
        if state.authorization_key.is_some() {
            " (authorisation required)"
        } else {
            ""
        }
    );
    runtime
        .block_on(crate::server::serve(args.bind, state))
        .with_context(|| format!("cannot serve on {}", args.bind))?;
    Ok(ExitCode::SUCCESS)
}
