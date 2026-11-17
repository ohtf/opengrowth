// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

//! `opengrowth serve`: the HTTP API.

use std::net::SocketAddr;
use std::process::ExitCode;
use std::sync::Arc;

use anyhow::Context as _;
use opengrowth_data::ReferenceStore;
use opengrowth_server::AppState;

use super::Context;
use crate::paths;

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
}

/// Runs `serve` until interrupted.
pub fn run(context: &Context, args: &Args) -> anyhow::Result<ExitCode> {
    let dir = paths::reference_dir(context.data_dir.as_deref())?;
    let store = if dir.exists {
        ReferenceStore::load_dir(&dir.path)?
    } else {
        ReferenceStore::empty()
    };
    if !opengrowth_server::uk_who_ready(&store) {
        eprintln!(
            "warning: UK-WHO tables are not all present in {} ({}); calculations will return 503. \
             Import them with `opengrowth data import <DIR>`.",
            dir.path.display(),
            dir.source
        );
    }
    let state = Arc::new(AppState {
        store,
        authorization_key: args.authorization_key.clone().filter(|k| !k.is_empty()),
    });

    let mut runtime = tokio::runtime::Builder::new_multi_thread();
    runtime.enable_all();
    if let Some(workers) = args.workers {
        runtime.worker_threads(workers);
    }
    let runtime = runtime.build().context("cannot start the async runtime")?;
    eprintln!(
        "opengrowth {} listening on http://{}{}",
        opengrowth_core::ENGINE_VERSION,
        args.bind,
        if state.authorization_key.is_some() {
            " (authorisation required)"
        } else {
            ""
        }
    );
    runtime
        .block_on(opengrowth_server::serve(args.bind, state))
        .with_context(|| format!("cannot serve on {}", args.bind))?;
    Ok(ExitCode::SUCCESS)
}
