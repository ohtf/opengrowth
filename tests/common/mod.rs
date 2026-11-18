// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Helpers for the conformance tests in `tests/`. See `conformance/README.md`.

#![allow(dead_code, reason = "each test binary uses a subset")]

use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;

/// Environment variable naming a directory of reference tables in the
/// `rcpchgrowth` `data_tables` layout.
pub const DATA_DIR_ENV: &str = "OPENGROWTH_REFERENCE_DATA_DIR";

/// The repository's `conformance/` directory.
#[must_use]
pub fn conformance_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("conformance")
}

/// Reads and parses a JSON file under `conformance/`.
///
/// # Panics
///
/// If the file is missing or malformed: a conformance input that cannot be
/// read is a test failure, not something to skip.
#[must_use]
pub fn read_json<T: DeserializeOwned>(relative: &str) -> T {
    let path = conformance_dir().join(relative);
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("cannot parse {}: {e}", path.display()))
}

/// A golden file written by `conformance/generators/_common.py`.
#[derive(Debug, serde::Deserialize)]
pub struct Golden<T> {
    /// Generator script.
    pub generator: String,
    /// Versions of the reference environment.
    pub environment: serde_json::Value,
    /// The cases.
    pub cases: T,
}

/// Running statistics for differences between two implementations.
#[derive(Debug, Default, Clone, Copy)]
pub struct Agreement {
    /// Cases compared.
    pub compared: usize,
    /// Cases agreeing to the last bit.
    pub bit_exact: usize,
    /// Largest absolute difference.
    pub max_abs: f64,
    /// Largest relative difference, where the expected value is non-zero.
    pub max_rel: f64,
    /// Largest difference in units in the last place.
    pub max_ulps: u64,
}

impl Agreement {
    /// Records one comparison.
    pub fn record(&mut self, actual: f64, expected: f64) {
        self.compared += 1;
        if actual.to_bits() == expected.to_bits() {
            self.bit_exact += 1;
            return;
        }
        let abs = (actual - expected).abs();
        self.max_abs = self.max_abs.max(abs);
        if expected != 0.0 {
            self.max_rel = self.max_rel.max(abs / expected.abs());
        }
        self.max_ulps = self.max_ulps.max(ulps_between(actual, expected));
    }

    /// One-line summary.
    #[must_use]
    pub fn summary(&self) -> String {
        let percent = if self.compared == 0 {
            0.0
        } else {
            100.0 * self.bit_exact as f64 / self.compared as f64
        };
        format!(
            "{} compared, {} bit-exact ({percent:.2}%), max |diff| {:.3e}, max rel {:.3e}, max {} ulp",
            self.compared, self.bit_exact, self.max_abs, self.max_rel, self.max_ulps
        )
    }
}

/// Distance in representable doubles between two finite values.
#[must_use]
pub fn ulps_between(a: f64, b: f64) -> u64 {
    fn ordered(x: f64) -> i64 {
        let bits = x.to_bits() as i64;
        if bits < 0 { i64::MIN - bits } else { bits }
    }
    ordered(a).abs_diff(ordered(b))
}

/// Removes every `"provenance": <value>` member and its separating comma,
/// exactly as `conformance/generators/http_uk_who.py` does, so bodies can be
/// compared outside the one designed difference.
#[must_use]
pub fn mask_provenance(body: &str) -> String {
    const KEY: &str = "\"provenance\":";
    let mut out = body.to_owned();
    let mut start = 0;
    while let Some(found) = out[start..].find(KEY) {
        let i = start + found;
        if i == 0 || !matches!(out.as_bytes()[i - 1], b'{' | b',') {
            start = i + KEY.len();
            continue;
        }
        let value_start = i + KEY.len();
        let mut stream = serde_json::Deserializer::from_str(&out[value_start..])
            .into_iter::<serde::de::IgnoredAny>();
        if stream.next().is_none() {
            break;
        }
        let end = value_start + stream.byte_offset();
        if out.as_bytes().get(end) == Some(&b',') {
            out.replace_range(i..=end, "");
        } else if out.as_bytes()[i - 1] == b',' {
            out.replace_range(i - 1..end, "");
        } else {
            out.replace_range(i..end, "");
        }
        start = i.saturating_sub(1);
    }
    out
}

/// Lowercase hex SHA-256.
#[must_use]
pub fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::Digest;
    sha2::Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
