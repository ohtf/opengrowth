// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Where reference tables are found, resolved deterministically.
//!
//! In order, the first that applies wins:
//!
//! 1. `--data-dir`, or `$OPENGROWTH_REFERENCE_DATA_DIR` (clap merges the two).
//!    Must exist.
//! 2. `$OPENGROWTH_DATA_HOME/reference-data`
//! 3. `$XDG_DATA_HOME/opengrowth/reference-data`
//! 4. `~/.local/share/opengrowth/reference-data`
//!
//! Options 2 to 4 are where `opengrowth data import` writes, and may not
//! exist yet, in which case no tables are loaded.

use std::convert::Infallible;
use std::env;
use std::path::{Path, PathBuf};

use anyhow::{Context as _, bail};
use serde::Serialize;

/// Directory of tables to use directly.
pub const DATA_DIR_ENV: &str = "OPENGROWTH_REFERENCE_DATA_DIR";
/// Root of OpenGrowth's own data directory.
pub const DATA_HOME_ENV: &str = "OPENGROWTH_DATA_HOME";

/// A resolved reference-data directory and the rule that chose it.
#[derive(Debug, Clone, Serialize)]
pub struct DataDir {
    /// The directory.
    pub path: PathBuf,
    /// Which rule chose it, for `data status` and error messages.
    pub source: &'static str,
    /// Whether it exists.
    pub exists: bool,
}

/// Resolves the directory tables are read from.
pub fn reference_dir(explicit: Option<&Path>) -> anyhow::Result<DataDir> {
    if let Some(path) = explicit {
        if !path.is_dir() {
            bail!(
                "reference data directory {} does not exist (from --data-dir or ${DATA_DIR_ENV})",
                path.display()
            );
        }
        return Ok(DataDir {
            path: path.to_owned(),
            source: "--data-dir or $OPENGROWTH_REFERENCE_DATA_DIR",
            exists: true,
        });
    }
    import_dir()
}

/// Resolves the directory `data import` writes to.
pub fn import_dir() -> anyhow::Result<DataDir> {
    let (home, source) = if let Some(home) = env_path(DATA_HOME_ENV) {
        (home, "$OPENGROWTH_DATA_HOME/reference-data")
    } else if let Some(xdg) = env_path("XDG_DATA_HOME") {
        (
            xdg.join("opengrowth"),
            "$XDG_DATA_HOME/opengrowth/reference-data",
        )
    } else {
        let home =
            env_path("HOME").context("cannot find a home directory; set OPENGROWTH_DATA_HOME")?;
        (
            home.join(".local/share/opengrowth"),
            "~/.local/share/opengrowth/reference-data",
        )
    };
    let path = home.join("reference-data");
    let exists = path.is_dir();
    Ok(DataDir {
        path,
        source,
        exists,
    })
}

fn env_path(name: &str) -> Option<PathBuf> {
    env::var_os(name)
        .filter(|v| !v.is_empty())
        .map(|v| expand_tilde(Path::new(&v)))
}

/// Expands a leading `~` or `~/` to the home directory.
pub fn expand_tilde(path: &Path) -> PathBuf {
    let Ok(rest) = path.strip_prefix("~") else {
        return path.to_owned();
    };
    match env::var_os("HOME") {
        Some(home) => PathBuf::from(home).join(rest),
        None => path.to_owned(),
    }
}

/// clap value parser for path arguments: expands `~` so quoted paths work.
#[allow(clippy::unnecessary_wraps, reason = "clap value parsers return Result")]
pub fn tilde_path(value: &str) -> Result<PathBuf, Infallible> {
    Ok(expand_tilde(Path::new(value)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tilde_expansion_only_touches_a_leading_tilde() {
        let home = PathBuf::from(env::var_os("HOME").unwrap());
        assert_eq!(expand_tilde(Path::new("~/x")), home.join("x"));
        assert_eq!(expand_tilde(Path::new("~")), home);
        assert_eq!(expand_tilde(Path::new("/a/~b")), PathBuf::from("/a/~b"));
        assert_eq!(
            expand_tilde(Path::new("~other/x")),
            PathBuf::from("~other/x")
        );
    }
}
