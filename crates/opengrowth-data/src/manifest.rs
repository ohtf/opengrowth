// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The checksum manifest: every table opengrowth knows how to use, where it
//! comes from, and what it is allowed to contain.
//!
//! The manifest holds metadata and SHA-256 digests only, never table values.
//! A file is used only if its digest matches exactly, so an edited, truncated,
//! or substituted table is refused rather than silently used (SAFETY.md,
//! "Wrong or corrupted user-supplied reference data").
//!
//! Digests are of the files distributed in the `rcpchgrowth` 4.6.5 wheel on
//! PyPI, which are byte-identical to the same files in the upstream
//! repository at that release. Users who hold the data under their own
//! terms can point opengrowth at that package's `data_tables` directory.

use opengrowth_core::TableId;

/// Redistribution status of a table, as far as it is currently known.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Terms {
    /// Redistribution is not known to be permitted. Never committed to this
    /// repository or included in a public artefact (`spec/queries.md` Q1).
    Restricted {
        /// Who owns the data.
        owner: &'static str,
        /// Why it is restricted.
        note: &'static str,
    },
    /// Terms not yet recorded. Treated as restricted until `R2` records them.
    Unrecorded {
        /// What is known so far.
        note: &'static str,
    },
}

impl Terms {
    /// Short label for status output.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Restricted { .. } => "restricted",
            Self::Unrecorded { .. } => "terms unrecorded",
        }
    }
}

/// One expected source file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TableSource {
    /// Which table this file provides.
    pub id: TableId,
    /// Path relative to the reference-data directory, using `/`.
    pub file: &'static str,
    /// Lowercase hex SHA-256 of the exact file contents.
    pub sha256: &'static str,
    /// Human-readable description.
    pub title: &'static str,
    /// Where the verified copy was obtained.
    pub origin: &'static str,
    /// Redistribution status.
    pub terms: Terms,
}

const UK90_TERMS: Terms = Terms::Restricted {
    owner: "UK Research and Innovation (formerly the Medical Research Council)",
    note: "licence application pending; see spec/queries.md Q1",
};

const WHO_TERMS: Terms = Terms::Unrecorded {
    note: "described upstream as open data; terms to be recorded in R2",
};

const ORIGIN: &str = "rcpchgrowth 4.6.5 (PyPI), rcpchgrowth/data_tables";

/// Every table opengrowth can load.
pub const MANIFEST: &[TableSource] = &[
    TableSource {
        id: TableId::Uk90Preterm,
        file: "uk90_preterm.json",
        sha256: "fcd126301d8fb961e3006b828688022ce950f085b0ec1d877a40c8f6df21f3ef",
        title: "UK90 preterm and neonatal, 23 to 42 weeks gestation",
        origin: ORIGIN,
        terms: UK90_TERMS,
    },
    TableSource {
        id: TableId::Who2006Infant,
        file: "who/who_infants.json",
        sha256: "c23dff79e6d83e2d474ac6856d57e25e6e88350b1a08a0a0268b7c9595c9026c",
        title: "WHO 2006 growth standard, birth to 2 years, daily",
        origin: ORIGIN,
        terms: WHO_TERMS,
    },
    TableSource {
        id: TableId::Who2006Child,
        file: "who/who_children.json",
        sha256: "cf92ceeae7a519c26b85681c38585908277da168dad700af262d4b296b007e6d",
        title: "WHO 2006 growth standard, 2 to 5 years, daily",
        origin: ORIGIN,
        terms: WHO_TERMS,
    },
    TableSource {
        id: TableId::Uk90Child,
        file: "uk90_child.json",
        sha256: "4a6f4e9f258e57682224bdbb03dec151219bb52c5dee181b483745f60fe87441",
        title: "UK90, 4 to 20 years",
        origin: ORIGIN,
        terms: UK90_TERMS,
    },
];

/// The manifest entry for a table.
///
/// # Panics
///
/// If a [`TableId`] has no manifest entry, which the registry test rules out.
#[must_use]
pub fn source(id: TableId) -> &'static TableSource {
    MANIFEST
        .iter()
        .find(|s| s.id == id)
        .expect("every TableId has a manifest entry")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_every_table_has_provenance_terms_and_a_digest() {
        for id in [
            TableId::Uk90Preterm,
            TableId::Who2006Infant,
            TableId::Who2006Child,
            TableId::Uk90Child,
        ] {
            let entry = source(id);
            assert_eq!(entry.sha256.len(), 64, "{id}");
            assert!(
                entry
                    .sha256
                    .bytes()
                    .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()),
                "{id}"
            );
            assert!(!entry.title.is_empty() && !entry.origin.is_empty(), "{id}");
            assert!(
                !entry.file.starts_with('/') && !entry.file.contains(".."),
                "{id}"
            );
        }
        let mut ids: Vec<_> = MANIFEST.iter().map(|s| s.id).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), MANIFEST.len(), "duplicate manifest entries");
    }
}
