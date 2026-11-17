// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The OpenGrowth calculation engine.
//!
//! Computes SDS and centiles for children's growth measurements using the
//! LMS method against published growth references. It is a Rust
//! reimplementation of the calculations in RCPCH's `rcpchgrowth` Python
//! package; see `NOTICE.md` and `spec/standard.md`.
//!
//! The engine is a leaf: no clock, no file or network I/O, no global state,
//! no async runtime. Reference tables are supplied by the caller through
//! [`data::ReferenceData`].
//!
//! **Not for clinical use.** See `SAFETY.md`.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod data;
pub mod dates;
pub mod interpolation;
pub mod lms;
pub mod normal;
pub mod round;
pub mod sds;
pub mod table;
pub mod uk_who;
pub mod vocabulary;

pub use data::{DataError, ReferenceData, TableId};
pub use dates::Gestation;
pub use sds::{CalcError, SdsResult, centile, sds_for_measurement};
pub use table::{LmsPoint, LmsTable};
pub use vocabulary::{GrowthReference, MeasurementMethod, Sex};

/// The engine name reported in provenance (`spec/provenance.md` section 3).
pub const ENGINE_NAME: &str = "opengrowth";

/// The engine version reported in provenance.
pub const ENGINE_VERSION: &str = env!("CARGO_PKG_VERSION");
