// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-FileCopyrightText: Royal College of Paediatrics and Child Health
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The growth references, each one self-contained [`Definition`]
//! (`spec/standard.md` section 7.8).
//!
//! A definition holds what opengrowth knows about one reference: which table
//! serves an age, and where it has none; how its SDS is calculated; how it
//! corrects ages for gestation; which centile collection describes a
//! measurement; how its chart is drawn; and how upstream's API names and
//! serves it. The engine, charts, and server read definitions rather than
//! branching on the reference, so adding a reference means adding a module
//! here, its [`GrowthReference`] value, and its tables.

pub mod cdc;
pub mod trisomy_21;
pub mod trisomy_21_aap;
pub mod turner;
pub mod uk_who;
pub mod who;

use crate::advice::CentileFormat;
use crate::data::TableId;
use crate::table::Interpolation;
use crate::vocabulary::{GrowthReference, MeasurementMethod, Sex};

/// Everything opengrowth knows about one growth reference.
#[derive(Debug)]
pub struct Definition {
    /// The reference.
    pub reference: GrowthReference,
    /// Its name in the command-line tool's output.
    pub label: &'static str,
    /// The table for a measurement at a decimal age, or why the reference
    /// has none. `youngest_at_boundary` chooses the younger table at a join
    /// between two tables, so that a chart line reaches the join; SDS
    /// calculations never ask for it.
    pub select: fn(
        age: f64,
        method: MeasurementMethod,
        sex: Sex,
        youngest_at_boundary: bool,
    ) -> Result<TableId, Absence>,
    /// How its SDS is calculated.
    pub sds: SdsRules,
    /// How ages are corrected for gestation at birth.
    pub correction: Correction,
    /// If set, a corrected age before term gets no SDS, and this message
    /// says why.
    pub before_term: Option<&'static str>,
    /// The centile collection that describes a measurement.
    pub centile_format: fn(MeasurementMethod) -> CentileFormat,
    /// How its chart is drawn.
    pub chart: ChartRules,
    /// The age, in years, at which mid-parental height compares the
    /// parents' heights.
    pub adult_age: f64,
    /// How upstream's API serves it.
    pub api: Api,
}

/// Why a reference has no data for a measurement at an age. The message is
/// upstream's `LookupError` text, which reaches API responses unchanged.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("{message}")]
pub struct Absence {
    /// The reference.
    pub reference: GrowthReference,
    /// Upstream's message.
    pub message: &'static str,
}

/// How a reference calculates an SDS.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SdsRules {
    /// How SDS lookups interpolate between table rows. Charts, fictional
    /// children, and % median BMI always use the default, as upstream does.
    pub interpolation: Interpolation,
    /// The method.
    pub method: SdsMethod,
    /// Whether, at exactly 2 years, the lookup age is clamped into the
    /// selected daily table, whose first row is a day after the second
    /// birthday (upstream `_age_for_lms_lookup`).
    pub clamp_at_two_years: bool,
}

/// The method that turns a measurement into an SDS.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SdsMethod {
    /// The LMS method.
    Lms,
    /// WHO's restricted extension of the LMS method beyond 3 SD, for weight
    /// and BMI at any age and every measurement under 5 years.
    WhoRestricted,
    /// The CDC extended BMI-for-age method above the 95th centile, for BMI.
    CdcExtendedBmi,
}

/// How a reference corrects ages for gestation at birth.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Correction {
    /// Every gestation other than exactly 40 weeks, at every age.
    Always,
    /// WHO and CDC: 37 to 42 weeks count as term and are not corrected, and
    /// correction stops once the corrected age reaches 2 years.
    TermRule,
}

/// How a reference's chart is drawn: upstream's chart builder for it.
#[derive(Debug, Clone, Copy)]
pub struct ChartRules {
    /// Its segments, in upstream's order.
    pub segments: &'static [Segment],
    /// Whether the Cole nine-centile collection is drawn at SDS rounded to
    /// the nearest two-thirds. Only UK-WHO's builder does this.
    pub cole_rounding: bool,
    /// How a line's SDS is rounded.
    pub line_sds: LineSds,
    /// Whether a line whose SDS overflows is left out, rather than failing
    /// the chart. Only Turner's builder does this.
    pub drops_overflowing_lines: bool,
}

/// How a chart builder rounds a line's SDS, which also decides when a
/// line's SDS overflows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineSds {
    /// `round(z * 100) / 100`, which overflows once 100 times the SDS is not
    /// finite.
    Hundredths,
    /// `round(z, 2)`, as upstream's Trisomy 21 builder does, which overflows
    /// only at infinity.
    Rounded,
}

/// One segment of a chart: its name, its age grid for a measurement, and
/// the ages at which it uses the younger table.
#[derive(Debug, Clone, Copy)]
pub struct Segment {
    /// Upstream's name for the segment, which keys the chart response.
    pub name: &'static str,
    /// The ages its lines are calculated at.
    pub ages: fn(MeasurementMethod) -> &'static [f64],
    /// Whether an age is a join at which this segment uses the younger table.
    pub youngest_at: fn(f64) -> bool,
}

/// How upstream's API serves a reference.
#[derive(Debug, Clone, Copy)]
pub struct Api {
    /// The route prefix, without slashes: the reference's value, except
    /// `turner` for Turner syndrome.
    pub prefix: &'static str,
    /// What chart error messages call the reference.
    pub chart_name: &'static str,
    /// What fictional-child error messages call the reference.
    pub fictional_child_name: &'static str,
    /// The one sex and measurement the reference covers, if it covers only
    /// one, and the messages for other requests.
    pub only: Option<Only>,
    /// Whether calculations receive the request's bone age. Upstream's AAP
    /// Trisomy 21 routes drop it.
    pub bone_age: bool,
}

/// The one sex and measurement a reference covers, and the messages its
/// routes refuse other requests with, before calculating anything.
#[derive(Debug, Clone, Copy)]
pub struct Only {
    /// The sex.
    pub sex: Sex,
    /// The measurement.
    pub method: MeasurementMethod,
    /// Calculation of another sex: a `value_error` at `sex`.
    pub calculation_sex: &'static str,
    /// Calculation of another measurement: a `value_error` at
    /// `measurement_method`.
    pub calculation_method: &'static str,
    /// Chart coordinates of either.
    pub chart: &'static str,
    /// Fictional child data of another sex.
    pub fictional_child_sex: &'static str,
    /// Fictional child data of another measurement.
    pub fictional_child_method: &'static str,
}

/// Every definition, in the order upstream's server mounts their routes.
pub static ALL: [&Definition; 6] = [
    &uk_who::DEFINITION,
    &turner::DEFINITION,
    &trisomy_21::DEFINITION,
    &trisomy_21_aap::DEFINITION,
    &cdc::DEFINITION,
    &who::DEFINITION,
];

/// The definition of a reference.
#[must_use]
pub fn definition(reference: GrowthReference) -> &'static Definition {
    match reference {
        GrowthReference::UkWho => &uk_who::DEFINITION,
        GrowthReference::Who => &who::DEFINITION,
        GrowthReference::Cdc => &cdc::DEFINITION,
        GrowthReference::TurnersSyndrome => &turner::DEFINITION,
        GrowthReference::Trisomy21 => &trisomy_21::DEFINITION,
        GrowthReference::Trisomy21Aap => &trisomy_21_aap::DEFINITION,
    }
}

/// The Cole nine-centile collection, which describes every measurement in
/// every reference but CDC.
const fn cole_nine_centiles(_: MeasurementMethod) -> CentileFormat {
    CentileFormat::ColeNineCentiles
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_reference_has_its_own_definition_once() {
        assert_eq!(ALL.len(), GrowthReference::ALL.len());
        for reference in GrowthReference::ALL {
            assert_eq!(definition(reference).reference, reference);
            assert_eq!(ALL.iter().filter(|d| d.reference == reference).count(), 1);
        }
    }
}
