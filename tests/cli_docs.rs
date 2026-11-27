// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

#![cfg(feature = "cli")]

//! The command-line reference pages in `docs/cli/` are the program's current
//! help, with no page left from a removed command (`R56`).

use std::collections::BTreeSet;
use std::path::Path;

use opengrowth::cli::reference::{DIRECTORY, pages};

#[test]
fn the_cli_pages_are_current() {
    let docs = Path::new(env!("CARGO_MANIFEST_DIR")).join("docs");
    let pages = pages();
    let mut stale: Vec<String> = pages
        .iter()
        .filter(|(path, contents)| {
            std::fs::read_to_string(docs.join(path)).ok().as_ref() != Some(contents)
        })
        .map(|(path, _)| path.clone())
        .collect();
    let expected: BTreeSet<&String> = pages.iter().map(|(path, _)| path).collect();
    for entry in std::fs::read_dir(docs.join(DIRECTORY)).expect("docs/cli") {
        let path = format!(
            "{DIRECTORY}/{}",
            entry.expect("an entry").file_name().to_string_lossy()
        );
        if !expected.contains(&path) {
            stale.push(path);
        }
    }
    assert!(stale.is_empty(), "out of date: {stale:?}; run s/cli-docs");
}

#[test]
fn every_command_has_a_page() {
    let paths: Vec<String> = pages().into_iter().map(|(path, _)| path).collect();
    for command in ["calculate", "data", "serve", "openapi", "completions"] {
        assert!(
            paths.contains(&format!("{DIRECTORY}/{command}.md")),
            "{command}"
        );
    }
}
