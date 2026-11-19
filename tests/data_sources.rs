// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Publication source files in `data/sources/` match the checksums recorded
//! beside them in `SHA256SUMS`, and every source file is recorded, so a
//! source cannot change, or be added, without the change showing.

mod common;

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

/// Files in a source directory that are opengrowth's own, not the source's.
const OWN_FILES: [&str; 2] = ["SHA256SUMS", "SOURCE.md"];

#[test]
fn source_files_match_their_recorded_checksums() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/sources");
    let mut checked = 0;
    for dir in fs::read_dir(&root).expect("data/sources") {
        let dir = dir.expect("directory entry").path();
        let sums = fs::read_to_string(dir.join("SHA256SUMS"))
            .unwrap_or_else(|e| panic!("{}: {e}", dir.join("SHA256SUMS").display()));
        let mut listed = BTreeSet::new();
        for line in sums.lines() {
            let (digest, name) = line
                .split_once("  ")
                .unwrap_or_else(|| panic!("{}: not sha256sum output: {line}", dir.display()));
            let bytes = fs::read(dir.join(name))
                .unwrap_or_else(|e| panic!("{}: {e}", dir.join(name).display()));
            assert_eq!(
                common::sha256_hex(&bytes),
                digest,
                "{}",
                dir.join(name).display()
            );
            listed.insert(name.to_owned());
            checked += 1;
        }
        for entry in fs::read_dir(&dir).expect("source directory") {
            let name = entry
                .expect("entry")
                .file_name()
                .to_string_lossy()
                .into_owned();
            assert!(
                listed.contains(&name) || OWN_FILES.contains(&name.as_str()),
                "{} is not recorded in SHA256SUMS",
                dir.join(&name).display()
            );
        }
    }
    assert!(checked > 0, "no source files found");
}
