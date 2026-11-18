// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The interface between the engine and reference tables.
//!
//! The engine never reads files. A caller supplies tables through
//! [`ReferenceData`], whether they are embedded in the binary or imported by
//! the user (`spec/standard.md` section 7.2). This keeps the engine a leaf
//! crate and lets tests inject synthetic tables.

use crate::table::LmsTable;
use crate::vocabulary::{MeasurementMethod, Sex};

/// A source data table, identified by what it contains rather than by file name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TableId {
    /// UK90 preterm and neonatal, 23 to 42 weeks gestation.
    Uk90Preterm,
    /// WHO 2006 standard, birth to 2 years (daily).
    Who2006Infant,
    /// WHO 2006 standard, 2 to 5 years (daily).
    Who2006Child,
    /// UK90, 4 to 20 years.
    Uk90Child,
    /// WHO 2007 growth reference, 5 to 19 years (monthly).
    Who2007Child,
    /// The earlier WHO 2006 infant table, birth to 2 years, which the CDC
    /// reference uses for height and weight.
    Who2006InfantPre2025,
    /// CDC infant head circumference (with height and weight), birth to 3 years.
    CdcInfant,
    /// CDC 2000, 2 to 20 years, with the extended-BMI sigma.
    CdcChild,
}

impl TableId {
    /// Stable machine name, used in provenance and error messages.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Uk90Preterm => "uk90-preterm",
            Self::Who2006Infant => "who-2006-infant",
            Self::Who2006Child => "who-2006-child",
            Self::Uk90Child => "uk90-child",
            Self::Who2007Child => "who-2007-child",
            Self::Who2006InfantPre2025 => "who-2006-infant-pre-2025",
            Self::CdcInfant => "cdc-infant",
            Self::CdcChild => "cdc-child",
        }
    }
}

impl std::fmt::Display for TableId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A table the engine asked for is not available.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DataError {
    /// The table has not been embedded or imported. The engine never
    /// substitutes another reference for a missing one.
    #[error("reference data table `{table}` is not available")]
    Unavailable {
        /// The missing table.
        table: TableId,
    },
    /// The table exists but has no rows for this measurement and sex.
    #[error("reference data table `{table}` has no {method} data for {sex}s")]
    MissingMeasurement {
        /// The table.
        table: TableId,
        /// Requested measurement.
        method: MeasurementMethod,
        /// Requested sex.
        sex: Sex,
    },
}

/// A source of LMS tables.
pub trait ReferenceData {
    /// The table for one source, measurement method, and sex.
    ///
    /// # Errors
    ///
    /// If the table has not been loaded, or has no data for this measurement and sex.
    fn table(
        &self,
        id: TableId,
        method: MeasurementMethod,
        sex: Sex,
    ) -> Result<&LmsTable, DataError>;
}
