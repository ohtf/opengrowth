// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::process::ExitCode;

fn main() -> ExitCode {
    reset_sigpipe();
    match opengrowth_cli::run(std::env::args_os()) {
        Ok(code) => code,
        Err(error) => {
            eprintln!("error: {error:#}");
            ExitCode::FAILURE
        }
    }
}

/// Restore the default SIGPIPE action so output piped into `head` or `less`
/// ends quietly instead of panicking on a broken pipe.
#[cfg(unix)]
#[allow(
    unsafe_code,
    reason = "a single libc call with no Rust-side invariants"
)]
fn reset_sigpipe() {
    unsafe extern "C" {
        fn signal(signum: i32, handler: usize) -> usize;
    }
    const SIGPIPE: i32 = 13;
    const SIG_DFL: usize = 0;
    // SAFETY: setting a signal to its default disposition has no preconditions.
    unsafe {
        signal(SIGPIPE, SIG_DFL);
    }
}

#[cfg(not(unix))]
fn reset_sigpipe() {}
