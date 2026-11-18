// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

//! opengrowth: growth calculations for children.
//!
//! Calculates SDS and centiles for children's growth measurements using the
//! LMS method against published growth references. It is a Rust
//! reimplementation of the calculations in RCPCH's `rcpchgrowth` Python
//! package; see `NOTICE.md` and `spec/standard.md`.
//!
//! One crate, three layers:
//!
//! - **The engine** (every module except the three below) is a leaf: no
//!   clock, no file or network I/O, no global state, no async runtime.
//!   Reference tables reach it through [`data::ReferenceData`]. With
//!   `default-features = false` this is all you get, plus [`store`].
//! - [`store`] loads and verifies reference tables: embedded in the binary
//!   (feature `embedded-data`) or imported from files.
//! - `server` (feature `server`) and `cli` (feature `cli`) are the HTTP API
//!   and the command-line tool.
//!
//! **Not for clinical use.** See `SAFETY.md`.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod advice;
pub mod cdc;
pub mod chart;
pub mod data;
pub mod dates;
pub mod fictional_child;
pub mod interpolation;
pub mod lms;
pub mod measurement;
pub mod mid_parental_height;
pub mod normal;
pub mod pyfmt;
pub mod round;
pub mod sds;
pub mod syndromes;
pub mod table;
pub mod uk_who;
pub mod vocabulary;
pub mod who;

pub mod store;

#[cfg(feature = "server")]
pub mod server;

#[cfg(feature = "cli")]
#[doc(hidden)]
pub mod cli;

pub use data::{DataError, ReferenceData, TableId};
pub use dates::Gestation;
pub use measurement::{MeasureError, Measurement, MeasurementInput, measure};
pub use sds::{CalcError, SdsResult, centile, sds_for_measurement};
pub use table::{LmsPoint, LmsTable};
pub use vocabulary::{GrowthReference, MeasurementMethod, Sex};

/// The engine name reported in provenance (`spec/provenance.md` section 3).
pub const ENGINE_NAME: &str = "opengrowth";

/// The engine version reported in provenance.
pub const ENGINE_VERSION: &str = env!("CARGO_PKG_VERSION");
