// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Writes the command-line reference pages under `docs/cli/` from the
//! program's own help (`R56`), and removes any page left from a command that
//! no longer exists. Run through `s/cli-docs`.

use std::collections::BTreeSet;
use std::path::Path;

use opengrowth::cli::reference::{DIRECTORY, pages};

fn main() -> std::io::Result<()> {
    let docs = Path::new(env!("CARGO_MANIFEST_DIR")).join("docs");
    let directory = docs.join(DIRECTORY);
    std::fs::create_dir_all(&directory)?;
    let mut written = BTreeSet::new();
    for (path, contents) in pages() {
        std::fs::write(docs.join(&path), contents)?;
        written.insert(path);
    }
    for entry in std::fs::read_dir(&directory)? {
        let name = entry?.file_name().to_string_lossy().into_owned();
        if !written.contains(&format!("{DIRECTORY}/{name}")) {
            std::fs::remove_file(directory.join(&name))?;
            eprintln!("removed docs/{DIRECTORY}/{name}");
        }
    }
    eprintln!("wrote {} pages in docs/{DIRECTORY}", written.len());
    Ok(())
}
