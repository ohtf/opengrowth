// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Loading, verifying, and importing opengrowth reference tables.
//!
//! Tables come from two places (`spec/standard.md` section 7.2):
//!
//! - **Embedded** in the binary from `data/` (feature `embedded-data`, on by
//!   default), verified against the manifest the first time they are used.
//! - **Imported** files in a data directory, which replace the embedded
//!   table with the same identity. This lets a deployment pin or update a
//!   table without a new binary.
//!
//! Either way, the bytes must match the SHA-256 digest in
//! [`manifest::MANIFEST`] and pass the [`crate::LmsTable`] invariants, or the
//! table is refused.

pub mod manifest;
mod parse;

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::{DataError, LmsTable, MeasurementMethod, ReferenceData, Sex, TableId};
use sha2::{Digest, Sha256};

pub use manifest::{MANIFEST, TableSource, Terms};

/// A reference-data file could not be used.
#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    /// The file exists but could not be read or written.
    #[error("cannot access {path}: {source}")]
    Io {
        /// The file.
        path: PathBuf,
        /// Underlying error.
        source: io::Error,
    },
    /// The file's contents do not match the manifest.
    #[error(
        "{path} does not match the expected checksum (expected {expected}, found {actual}); \
         it has been modified, truncated, or comes from a different release, and will not be used"
    )]
    Checksum {
        /// The file.
        path: PathBuf,
        /// Digest in the manifest.
        expected: &'static str,
        /// Digest of the file.
        actual: String,
    },
    /// The file matches its checksum but is not in the expected format.
    /// This would indicate a manifest error, since the digest pins the content.
    #[error("{path} is not a valid reference table: {message}")]
    Format {
        /// The file.
        path: PathBuf,
        /// What was wrong.
        message: String,
    },
}

/// Whether one manifest file is present and usable in a directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileState {
    /// Not present.
    Missing,
    /// Present and matches the manifest digest.
    Verified,
    /// Present but its digest differs. It will not be used.
    ChecksumMismatch {
        /// Digest of the file found.
        actual: String,
    },
    /// Present but unreadable.
    Unreadable {
        /// Why.
        message: String,
    },
}

/// The state of one manifest file in a directory.
#[derive(Debug, Clone)]
pub struct FileStatus {
    /// The manifest entry.
    pub source: &'static TableSource,
    /// Where it was looked for.
    pub path: PathBuf,
    /// What was found.
    pub state: FileState,
}

/// Checks every manifest file in `dir` without loading any of them.
#[must_use]
pub fn inspect(dir: &Path) -> Vec<FileStatus> {
    MANIFEST
        .iter()
        .map(|source| {
            let path = dir.join(source.file);
            let state = match fs::read(&path) {
                Err(e) if e.kind() == io::ErrorKind::NotFound => FileState::Missing,
                Err(e) => FileState::Unreadable {
                    message: e.to_string(),
                },
                Ok(bytes) => {
                    let actual = sha256_hex(&bytes);
                    if actual == source.sha256 {
                        FileState::Verified
                    } else {
                        FileState::ChecksumMismatch { actual }
                    }
                }
            };
            FileStatus {
                source,
                path,
                state,
            }
        })
        .collect()
}

/// Where a loaded table came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Origin {
    /// Compiled into the binary.
    Embedded,
    /// Read from a file.
    Imported(PathBuf),
}

impl Origin {
    /// `embedded` or `imported`, as reported in provenance.
    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Embedded => "embedded",
            Self::Imported(_) => "imported",
        }
    }
}

/// A table file that has been verified and loaded.
#[derive(Debug, Clone)]
pub struct LoadedFile {
    /// The manifest entry it matched.
    pub source: &'static TableSource,
    /// Where it was loaded from.
    pub origin: Origin,
}

/// The embedded copy of a manifest file, if this build embeds it.
#[must_use]
pub fn embedded_bytes(id: TableId) -> Option<&'static [u8]> {
    #[cfg(feature = "embedded-data")]
    {
        Some(match id {
            TableId::Uk90Preterm => include_bytes!("../../data/uk90_preterm.json").as_slice(),
            TableId::Who2006Infant => include_bytes!("../../data/who/who_infants.json").as_slice(),
            TableId::Who2006Child => include_bytes!("../../data/who/who_children.json").as_slice(),
            TableId::Uk90Child => include_bytes!("../../data/uk90_child.json").as_slice(),
        })
    }
    #[cfg(not(feature = "embedded-data"))]
    {
        let _ = id;
        None
    }
}

/// Reference tables loaded from verified files.
#[derive(Debug, Default)]
pub struct ReferenceStore {
    tables: BTreeMap<parse::TableKey, LmsTable>,
    loaded: Vec<LoadedFile>,
}

impl ReferenceStore {
    /// A store with no tables. Every lookup reports the table as unavailable.
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }

    /// Every table this build embeds, verified.
    ///
    /// # Errors
    ///
    /// Only if an embedded table fails verification, which would mean the
    /// build itself is corrupt.
    pub fn embedded() -> Result<Self, LoadError> {
        let mut store = Self::empty();
        for source in MANIFEST {
            if let Some(bytes) = embedded_bytes(source.id) {
                let label = PathBuf::from(format!("embedded:data/{}", source.file));
                store.insert(
                    source,
                    Origin::Embedded,
                    verify_and_parse(source, &label, bytes)?,
                );
            }
        }
        Ok(store)
    }

    /// Loads every manifest file present in `dir`. Missing files are simply
    /// absent from the store. A present file that fails verification is an
    /// error, and nothing is loaded.
    ///
    /// # Errors
    ///
    /// If any present file is unreadable, has the wrong checksum, or does not parse.
    pub fn load_dir(dir: &Path) -> Result<Self, LoadError> {
        Self::empty().with_dir(dir)
    }

    /// The embedded tables, with any verified files in `dir` replacing them.
    /// `dir` may be absent or not exist.
    ///
    /// # Errors
    ///
    /// As [`Self::embedded`] and [`Self::load_dir`].
    pub fn standard(dir: Option<&Path>) -> Result<Self, LoadError> {
        let store = Self::embedded()?;
        match dir {
            Some(dir) if dir.is_dir() => store.with_dir(dir),
            _ => Ok(store),
        }
    }

    /// Adds the verified files present in `dir`, each replacing any table
    /// already loaded with the same identity.
    ///
    /// # Errors
    ///
    /// If any present file is unreadable, has the wrong checksum, or does not
    /// parse. Then nothing from `dir` is used.
    pub fn with_dir(mut self, dir: &Path) -> Result<Self, LoadError> {
        let mut found = Vec::new();
        for source in MANIFEST {
            let path = dir.join(source.file);
            let bytes = match fs::read(&path) {
                Err(e) if e.kind() == io::ErrorKind::NotFound => continue,
                Err(source) => return Err(LoadError::Io { path, source }),
                Ok(bytes) => bytes,
            };
            found.push((source, verify_and_parse(source, &path, &bytes)?, path));
        }
        for (source, tables, path) in found {
            self.insert(source, Origin::Imported(path), tables);
        }
        Ok(self)
    }

    fn insert(&mut self, source: &'static TableSource, origin: Origin, tables: Vec<parse::Keyed>) {
        self.tables.retain(|(id, _, _), _| *id != source.id);
        self.loaded.retain(|f| f.source.id != source.id);
        self.tables.extend(tables);
        self.loaded.push(LoadedFile { source, origin });
        self.loaded.sort_by_key(|f| f.source.id);
    }

    /// The files that were loaded.
    #[must_use]
    pub fn loaded(&self) -> &[LoadedFile] {
        &self.loaded
    }

    /// The loaded file that provides `id`, if any.
    #[must_use]
    pub fn file_for(&self, id: TableId) -> Option<&LoadedFile> {
        self.loaded.iter().find(|f| f.source.id == id)
    }
}

impl ReferenceData for ReferenceStore {
    fn table(
        &self,
        id: TableId,
        method: MeasurementMethod,
        sex: Sex,
    ) -> Result<&LmsTable, DataError> {
        if let Some(table) = self.tables.get(&(id, method, sex)) {
            return Ok(table);
        }
        if self.file_for(id).is_some() {
            Err(DataError::MissingMeasurement {
                table: id,
                method,
                sex,
            })
        } else {
            Err(DataError::Unavailable { table: id })
        }
    }
}

/// What import did with one manifest file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportAction {
    /// Copied into the destination.
    Imported,
    /// Already present at the destination with the right checksum.
    AlreadyPresent,
    /// Not found in the source directory.
    NotInSource,
}

/// The outcome of importing one manifest file.
#[derive(Debug, Clone)]
pub struct ImportOutcome {
    /// The manifest entry.
    pub source: &'static TableSource,
    /// What happened.
    pub action: ImportAction,
}

/// Copies every verified manifest file from `from` into `to`.
///
/// Every file found in `from` is fully verified (checksum and format) before
/// anything is written, so a bad file aborts the whole import. Each file is
/// written to a temporary name beside its destination and renamed into
/// place, so an interrupted import never leaves a partial table.
///
/// # Errors
///
/// If any source file fails verification, or a destination cannot be written.
pub fn import(from: &Path, to: &Path) -> Result<Vec<ImportOutcome>, LoadError> {
    let mut verified = Vec::new();
    let mut outcomes = Vec::new();
    for source in MANIFEST {
        let path = from.join(source.file);
        match fs::read(&path) {
            Err(e) if e.kind() == io::ErrorKind::NotFound => outcomes.push(ImportOutcome {
                source,
                action: ImportAction::NotInSource,
            }),
            Err(e) => return Err(LoadError::Io { path, source: e }),
            Ok(bytes) => {
                verify_and_parse(source, &path, &bytes)?;
                verified.push((source, bytes));
            }
        }
    }

    for (source, bytes) in verified {
        let dest = to.join(source.file);
        let already = fs::read(&dest).is_ok_and(|existing| sha256_hex(&existing) == source.sha256);
        if !already {
            write_atomically(&dest, &bytes)?;
        }
        outcomes.push(ImportOutcome {
            source,
            action: if already {
                ImportAction::AlreadyPresent
            } else {
                ImportAction::Imported
            },
        });
    }
    outcomes.sort_by_key(|o| MANIFEST.iter().position(|s| s.id == o.source.id));
    Ok(outcomes)
}

fn verify_and_parse(
    source: &'static TableSource,
    path: &Path,
    bytes: &[u8],
) -> Result<Vec<parse::Keyed>, LoadError> {
    let actual = sha256_hex(bytes);
    if actual != source.sha256 {
        return Err(LoadError::Checksum {
            path: path.to_owned(),
            expected: source.sha256,
            actual,
        });
    }
    parse::tables(source.id, bytes).map_err(|message| LoadError::Format {
        path: path.to_owned(),
        message,
    })
}

fn write_atomically(dest: &Path, bytes: &[u8]) -> Result<(), LoadError> {
    let io_error = |path: &Path| {
        let path = path.to_owned();
        move |source| LoadError::Io { path, source }
    };
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).map_err(io_error(parent))?;
    }
    let mut temp_name = dest.as_os_str().to_owned();
    temp_name.push(".partial");
    let temp = PathBuf::from(temp_name);
    fs::write(&temp, bytes).map_err(io_error(&temp))?;
    fs::File::open(&temp)
        .and_then(|f| f.sync_all())
        .map_err(io_error(&temp))?;
    fs::rename(&temp, dest).map_err(io_error(dest))
}

/// Lowercase hex SHA-256 of `bytes`.
#[must_use]
pub fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut hex = String::with_capacity(64);
    for byte in digest.iter() {
        write!(hex, "{byte:02x}").expect("writing to a String cannot fail");
    }
    hex
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A fresh, empty scratch directory unique to this test.
    fn scratch(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("opengrowth-data-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn an_empty_directory_loads_nothing_and_reports_tables_unavailable() {
        let dir = scratch("empty");
        let store = ReferenceStore::load_dir(&dir).unwrap();
        assert!(store.loaded().is_empty());
        assert_eq!(
            store.table(TableId::Uk90Child, MeasurementMethod::Height, Sex::Male),
            Err(DataError::Unavailable {
                table: TableId::Uk90Child
            })
        );
        assert!(inspect(&dir).iter().all(|s| s.state == FileState::Missing));
    }

    #[test]
    fn a_tampered_file_is_refused_by_load_inspect_and_import() {
        let source = scratch("tampered-src");
        let dest = scratch("tampered-dest");
        fs::write(source.join("uk90_child.json"), br#"{"measurement": {}}"#).unwrap();

        let err = ReferenceStore::load_dir(&source).unwrap_err();
        assert!(matches!(err, LoadError::Checksum { .. }), "{err}");

        let status = inspect(&source);
        let uk90 = status
            .iter()
            .find(|s| s.source.id == TableId::Uk90Child)
            .unwrap();
        assert!(matches!(uk90.state, FileState::ChecksumMismatch { .. }));

        assert!(matches!(
            import(&source, &dest),
            Err(LoadError::Checksum { .. })
        ));
        assert_eq!(
            fs::read_dir(&dest).unwrap().count(),
            0,
            "import wrote despite failing"
        );
    }

    #[cfg(feature = "embedded-data")]
    #[test]
    fn embedded_tables_verify_and_cover_every_manifest_entry() {
        let store = ReferenceStore::embedded().unwrap();
        assert_eq!(store.loaded().len(), MANIFEST.len());
        assert!(store.loaded().iter().all(|f| f.origin == Origin::Embedded));
        assert!(
            store
                .table(TableId::Uk90Child, MeasurementMethod::Height, Sex::Female)
                .is_ok()
        );
    }

    #[cfg(feature = "embedded-data")]
    #[test]
    fn a_tampered_directory_file_is_refused_even_over_embedded_data() {
        let dir = scratch("overlay");
        fs::write(dir.join("uk90_child.json"), br#"{"measurement": {}}"#).unwrap();
        assert!(matches!(
            ReferenceStore::standard(Some(&dir)),
            Err(LoadError::Checksum { .. })
        ));
    }

    #[test]
    fn digest_is_lowercase_hex() {
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
