// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Calculation provenance (`spec/provenance.md`): the growth reference a
//! result was calculated against, the engine that calculated it, and the
//! tables it read. It is a clinical safety control for RCPCH dGC hazard
//! #174: it lets a consumer see that a result was calculated against a
//! different reference from the chart it is plotted on.
//!
//! The engine records a [`Provenance`] in each result, from the reference
//! definition it used, so no surface ever states which reference it asked
//! for. Every surface writes it as a [`Record`], the one serialised form,
//! and adds only what the engine cannot know: the file and checksum of each
//! table, and the server's own identity.

use serde::Serialize;

use crate::data::TableId;
use crate::vocabulary::GrowthReference;

/// The commit a release build was stamped with, from `OPENGROWTH_COMMIT`,
/// or `"unknown"` in an unstamped build (`spec/provenance.md` section 4).
pub const COMMIT: &str = match option_env!("OPENGROWTH_COMMIT") {
    Some(commit) => commit,
    None => "unknown",
};

// A build stamped with anything else fails here, as upstream's schema
// refuses such a commit.
const _: () = assert!(
    is_commit(COMMIT),
    "OPENGROWTH_COMMIT must be a full 40-character lowercase commit hash"
);

/// Whether `value` can be reported as a commit: 40 lowercase hexadecimal
/// digits, or `"unknown"` (upstream's `^(?:[0-9a-f]{40}|unknown)$`).
#[must_use]
pub const fn is_commit(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() == 40 {
        let mut i = 0;
        while i < bytes.len() {
            if !matches!(bytes[i], b'0'..=b'9' | b'a'..=b'f') {
                return false;
            }
            i += 1;
        }
        return true;
    }
    let unknown = b"unknown";
    if bytes.len() != unknown.len() {
        return false;
    }
    let mut i = 0;
    while i < unknown.len() {
        if bytes[i] != unknown[i] {
            return false;
        }
        i += 1;
    }
    true
}

/// A program's identity in provenance: the engine's, or a server's.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Identity {
    /// Its name.
    pub name: &'static str,
    /// Its version.
    pub version: &'static str,
    /// The commit it was built from, or `"unknown"`.
    pub commit: &'static str,
}

/// This engine. Never the Python engine's name (`spec/provenance.md`
/// section 3).
pub const ENGINE: Identity = Identity {
    name: crate::ENGINE_NAME,
    version: crate::ENGINE_VERSION,
    commit: COMMIT,
};

/// What the engine records about a result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Provenance {
    /// The reference whose definition calculated the result.
    pub growth_reference: GrowthReference,
    /// The engine that calculated it.
    pub calculation_engine: Identity,
    /// Every table read, sorted, without duplicates: never the tables
    /// configured or expected, and empty if none was read.
    pub tables_read: Vec<TableId>,
}

impl Provenance {
    /// The provenance of a result calculated against `growth_reference`
    /// that read `tables_read`, in any order and with any repeats.
    #[must_use]
    pub fn new(growth_reference: GrowthReference, mut tables_read: Vec<TableId>) -> Self {
        tables_read.sort();
        tables_read.dedup();
        Self {
            growth_reference,
            calculation_engine: ENGINE,
            tables_read,
        }
    }
}

/// One table a result read, as provenance reports it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TableUsed {
    /// The table's name.
    pub table: &'static str,
    /// The file it was loaded from, relative to the data directory.
    pub file: &'static str,
    /// The SHA-256 of the verified file.
    pub sha256: &'static str,
    /// `embedded` or `imported`.
    pub source: &'static str,
}

/// Provenance as every surface writes it (`spec/provenance.md` section 1),
/// and the one place its shape is defined.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Record {
    /// The reference the result was calculated against.
    pub growth_reference: GrowthReference,
    /// The engine that calculated it.
    pub calculation_engine: Identity,
    /// The server that answered, if a server did.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_server: Option<Identity>,
    /// The tables read, with the file each came from.
    pub reference_data: Vec<TableUsed>,
}

impl Record {
    /// The record of `provenance`. `describe` gives the file behind each
    /// table read, which only the store knows, and `api_server` is the
    /// server writing it, if any.
    #[must_use]
    pub fn new(
        provenance: &Provenance,
        describe: impl Fn(TableId) -> Option<TableUsed>,
        api_server: Option<Identity>,
    ) -> Self {
        Self {
            growth_reference: provenance.growth_reference,
            calculation_engine: provenance.calculation_engine,
            api_server,
            reference_data: provenance
                .tables_read
                .iter()
                .filter_map(|&table| describe(table))
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_commit_is_a_full_lowercase_hash_or_unknown() {
        assert!(is_commit("unknown"));
        assert!(is_commit("29bb4e9e9d93a850a63d5233b927472f808fb9a8"));
        assert!(is_commit(COMMIT));
        for bad in [
            "not-a-commit",
            "",
            "Unknown",
            "29bb4e9",
            "29BB4E9E9D93A850A63D5233B927472F808FB9A8",
            "29bb4e9e9d93a850a63d5233b927472f808fb9a",
            "29bb4e9e9d93a850a63d5233b927472f808fb9a8 ",
            "29bb4e9e9d93a850a63d5233b927472f808fb9g8",
        ] {
            assert!(!is_commit(bad), "{bad:?}");
        }
    }

    #[test]
    fn the_engine_reports_its_own_name_and_version() {
        assert_eq!(ENGINE.name, "opengrowth");
        assert_ne!(ENGINE.name, "rcpch/rcpchgrowth-python");
        assert_eq!(ENGINE.version, env!("CARGO_PKG_VERSION"));
        assert!(!ENGINE.version.is_empty());
    }

    #[test]
    fn tables_are_sorted_once_each() {
        let provenance = Provenance::new(
            GrowthReference::UkWho,
            vec![
                TableId::Who2006Infant,
                TableId::Uk90Preterm,
                TableId::Who2006Infant,
            ],
        );
        assert_eq!(
            provenance.tables_read,
            [TableId::Uk90Preterm, TableId::Who2006Infant]
        );
    }

    #[test]
    fn a_record_lists_only_the_tables_read_in_the_contracts_key_order() {
        let provenance = Provenance::new(GrowthReference::TurnersSyndrome, vec![TableId::Turner]);
        let describe = |table: TableId| {
            Some(TableUsed {
                table: table.as_str(),
                file: "turner.json",
                sha256: "0",
                source: "embedded",
            })
        };
        let server = Identity {
            name: "server",
            ..ENGINE
        };
        let text =
            serde_json::to_string(&Record::new(&provenance, describe, Some(server))).unwrap();
        let keys = [
            "growth_reference",
            "calculation_engine",
            "api_server",
            "reference_data",
        ];
        let at: Vec<usize> = keys
            .iter()
            .map(|k| text.find(&format!("\"{k}\"")).unwrap())
            .collect();
        assert!(at.windows(2).all(|w| w[0] < w[1]), "{text}");
        assert!(
            text.starts_with(r#"{"growth_reference":"turners-syndrome","#),
            "{text}"
        );
        assert_eq!(text.matches(r#""table":"#).count(), 1, "{text}");

        let without_server =
            serde_json::to_string(&Record::new(&provenance, describe, None)).unwrap();
        assert!(!without_server.contains("api_server"), "{without_server}");
    }
}
