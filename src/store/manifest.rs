// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The checksum manifest: every table opengrowth knows how to use, where it
//! comes from, and what it is allowed to contain.
//!
//! The manifest holds metadata and SHA-256 digests. The values themselves
//! are in `data/`, embedded by the `embedded-data` feature, or imported.
//! A table is used only if its digest matches exactly, so an edited, truncated,
//! or substituted table is refused rather than silently used (SAFETY.md,
//! "Wrong or corrupted user-supplied reference data").
//!
//! Digests are of the files distributed in the `rcpchgrowth` 4.6.5 wheel on
//! PyPI, which are byte-identical to the same files in the upstream
//! repository at that release. Users who hold the data under their own
//! terms can point opengrowth at that package's `data_tables` directory.

use crate::TableId;

/// Redistribution status of a table, as far as it is currently known.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Terms {
    /// Redistribution or production use is not yet licensed (`spec/queries.md` Q1).
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
    note: "embedded for pre-alpha research and development only; production use awaits a UKRI licence (spec/queries.md Q1)",
};

const WHO_TERMS: Terms = Terms::Unrecorded {
    note: "described upstream as open data; terms to be recorded in R2",
};

const CDC_TERMS: Terms = Terms::Unrecorded {
    note: "CDC 2000 growth charts, a US federal government work (CDC National Center for Health Statistics); terms to be recorded in R2",
};

/// Tables opengrowth does not embed while `R2` has not recorded their terms.
/// They are verified and imported like any other: `opengrowth data import`
/// with a copy of `rcpchgrowth`'s `data_tables` (see `s/reference-tables`).
const IMPORT_ONLY: &str = "owner and terms not yet recorded (R2); not embedded, so import it";

const TURNER_TERMS: Terms = Terms::Unrecorded { note: IMPORT_ONLY };
const TRISOMY_21_TERMS: Terms = Terms::Unrecorded { note: IMPORT_ONLY };
const TRISOMY_21_AAP_TERMS: Terms = Terms::Unrecorded { note: IMPORT_ONLY };

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
    TableSource {
        id: TableId::Who2007Child,
        file: "who/who_2007_children.json",
        sha256: "0d1de5f761aa47020bfa9d194fa38dffd20eaa79e882a9436a75f322b49f6139",
        title: "WHO 2007 growth reference, 5 to 19 years, monthly",
        origin: ORIGIN,
        terms: WHO_TERMS,
    },
    TableSource {
        id: TableId::Who2006InfantPre2025,
        file: "who/pre_2025/who_infants.json",
        sha256: "4f8b3e3d1dc9eed9baf2ce9265ea836e0d033614c101f0325d4d7753996f044e",
        title: "WHO 2006 growth standard, birth to 2 years, the earlier table the CDC reference uses",
        origin: ORIGIN,
        terms: WHO_TERMS,
    },
    TableSource {
        id: TableId::CdcInfant,
        file: "cdc_infants.json",
        sha256: "c0b0556d48e33b6d6f3c6f5d455ed672739089a7fbf0075a257886da48a40b36",
        title: "CDC infant, birth to 3 years (head circumference to 3 years)",
        origin: ORIGIN,
        terms: CDC_TERMS,
    },
    TableSource {
        id: TableId::CdcChild,
        file: "cdc2-20.json",
        sha256: "6284ddf33ac305047f5ddb3f4d68c418c025dcb9e070ddcc3e23bff6880a58ef",
        title: "CDC 2000, 2 to 20 years, with the extended-BMI sigma",
        origin: ORIGIN,
        terms: CDC_TERMS,
    },
    TableSource {
        id: TableId::Turner,
        file: "turner.json",
        sha256: "d6f6951c24864cbcec4a07e5b64066cf5f74469dacf8390e8cba75a91aa78348",
        title: "UK Turner syndrome reference, girls' height, 1 to 20 years (Lyon, Preece and Grant, 1985)",
        origin: ORIGIN,
        terms: TURNER_TERMS,
    },
    TableSource {
        id: TableId::Trisomy21,
        file: "trisomy_21.json",
        sha256: "a04d1657c72a987baf4e222336092f1697b5160ada39ce069d424a15a7325b14",
        title: "UK Down syndrome reference, birth to 20 years (Styles, Cole, Dennis and Preece, 2002; BMI 2013)",
        origin: ORIGIN,
        terms: TRISOMY_21_TERMS,
    },
    TableSource {
        id: TableId::Trisomy21AapInfant,
        file: "trisomy_21_aap_infants.json",
        sha256: "ea90535df4c4aa8badbb58c6f39f5f957f8f1f202663aaa5c09af08034690781",
        title: "AAP Down syndrome growth charts, birth to 3 years (Zemel and others)",
        origin: ORIGIN,
        terms: TRISOMY_21_AAP_TERMS,
    },
    TableSource {
        id: TableId::Trisomy21AapChild,
        file: "trisomy_21_aap_children.json",
        sha256: "a388c04f0031812ad1078daca13acb5138185cf817e3a63400472950fd05c0cb",
        title: "AAP Down syndrome growth charts, 2 to 20 years (Zemel and others)",
        origin: ORIGIN,
        terms: TRISOMY_21_AAP_TERMS,
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
