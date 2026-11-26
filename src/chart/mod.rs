// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-FileCopyrightText: Royal College of Paediatrics and Child Health
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Chart coordinates: the centile lines of a growth chart (`R13`).
//!
//! Ported from `rcpchgrowth.chart_functions.create_uk_who_chart` and
//! `global_functions.generate_centile`. A line is the measurement at a fixed
//! SDS, calculated at each age in a fixed grid ([`ages`]), for each segment of
//! the reference. Ages where the reference has no data are left out of the
//! line, as upstream does.
//!
//! Each reference's segments, and the rules below that differ between
//! upstream's chart builders, are in its
//! [`ChartRules`](crate::references::ChartRules). Upstream behaviour kept
//! exactly, because it changes the coordinates:
//!
//! - For UK-WHO, the Cole nine-centile collection uses SDS rounded to the
//!   nearest two-thirds; every other collection, and every WHO and CDC
//!   collection (upstream's builders for those never use Cole's rounding),
//!   uses the exact inverse normal.
//! - At a join between segments, the line for the younger segment is
//!   calculated from the younger table, so it reaches the join. The test for
//!   the 42-week join compares against the literal `0.038329911`, which is
//!   just below the 42-week threshold, so that point stays in UK90 preterm.
//! - The requested SDS is rounded to 4 decimal places before use, ages and
//!   measurements are rounded to 4 places, and the line's SDS to 2.
//! - That rounding is NumPy's wherever the upstream value is a NumPy float
//!   ([`PyFloat`]): an SDS from SciPy's inverse normal, which is every
//!   collection except Cole's, whose SDS is a Python float except at the
//!   median; and measurements calculated from one, or from linearly
//!   interpolated LMS values.

pub mod ages;

use crate::advice::CentileFormat;
use crate::data::{DataError, ReferenceData, TableId};
use crate::normal;
use crate::provenance::Provenance;
use crate::references::{self, Definition, LineSds, Segment};
use crate::round::{PyFloat, python_round, python_round_int};
use crate::sds::{CalcError, measurement_from_sds};
use crate::vocabulary::{GrowthReference, MeasurementMethod, Sex};

/// Which centile lines to draw.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CentileSpec<'a> {
    /// A standard collection.
    Named(CentileFormat),
    /// Caller-chosen centiles, or SDS when `is_sds` is true.
    Custom {
        /// The values.
        values: &'a [f64],
        /// Whether `values` are SDS rather than centiles.
        is_sds: bool,
    },
}

/// One point on a line.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChartPoint {
    /// The line's label: its centile, or its SDS rounded to 3 places for SDS lines.
    pub label: f64,
    /// Decimal age, rounded to 4 places.
    pub x: f64,
    /// Measurement, rounded to 4 places; `None` where the LMS transform has
    /// no real answer at this SDS.
    pub y: Option<f64>,
}

/// One centile line within one segment.
#[derive(Debug, Clone, PartialEq)]
pub struct ChartLine {
    /// SDS, rounded to 2 places.
    pub sds: f64,
    /// Centile.
    pub centile: f64,
    /// The points, possibly none.
    pub data: Vec<ChartPoint>,
}

/// The lines for one segment of the reference, such as `uk90_preterm`.
#[derive(Debug, Clone, PartialEq)]
pub struct ChartSegment {
    /// Upstream segment name.
    pub name: &'static str,
    /// One line per requested centile, in request order.
    pub lines: Vec<ChartLine>,
}

/// A whole chart: its segments, and its provenance.
#[derive(Debug, Clone, PartialEq)]
pub struct Chart {
    /// One per segment of the reference, in upstream order.
    pub segments: Vec<ChartSegment>,
    /// The reference that drew it, the engine, and every table read.
    pub provenance: Provenance,
}

/// A chart could not be calculated.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ChartError {
    /// A requested centile or SDS gives an SDS that is not finite, or so
    /// large that 100 times it is not: centile 0, 100, or outside 0 to 100,
    /// or an SDS beyond about 1.8e306. Upstream's `round(z * 100)` raises an
    /// `OverflowError` or `ValueError`.
    #[error("centile line {0} has no finite SDS")]
    NonFinite(f64),
    /// A table is not loaded.
    #[error(transparent)]
    Data(#[from] DataError),
}

/// The centile lines for one reference, measurement, and sex (the upstream `create_chart`).
///
/// # Errors
///
/// See [`ChartError`].
pub fn chart(
    data: &dyn ReferenceData,
    reference: GrowthReference,
    method: MeasurementMethod,
    sex: Sex,
    spec: CentileSpec<'_>,
) -> Result<Chart, ChartError> {
    let definition = references::definition(reference);
    let rules = definition.chart;
    let segments = rules.segments;
    let (values, is_sds, cole) = match spec {
        CentileSpec::Named(format) => (
            format.centiles(),
            false,
            rules.cole_rounding && format == CentileFormat::ColeNineCentiles,
        ),
        CentileSpec::Custom { values, is_sds } => (values, is_sds, false),
    };

    let mut tables_read = Vec::new();
    let mut out = Vec::with_capacity(segments.len());
    for segment in segments {
        let mut lines = Vec::with_capacity(values.len());
        for &value in values {
            match line(
                data,
                definition,
                method,
                sex,
                segment,
                value,
                is_sds,
                cole,
                &mut tables_read,
            ) {
                Ok(line) => lines.push(line),
                // Upstream's Turner builder catches the overflow and leaves the line out.
                Err(ChartError::NonFinite(_)) if rules.drops_overflowing_lines => {}
                Err(error) => return Err(error),
            }
        }
        out.push(ChartSegment {
            name: segment.name,
            lines,
        });
    }
    Ok(Chart {
        segments: out,
        provenance: Provenance::new(definition.reference, tables_read),
    })
}

/// The SDS a centile is drawn at in the Cole nine-centile collection: the
/// exact SDS rounded to the nearest two-thirds (upstream `rounded_sds_for_centile`).
///
/// Upstream returns SciPy's NumPy float unchanged when it is 0, and
/// otherwise a Python float, the product of an `int` and `2 / 3`.
#[must_use]
pub fn cole_sds_for_centile(centile: f64) -> PyFloat {
    let sds = normal::ppf(centile / 100.0);
    if sds == 0.0 {
        return PyFloat::NumPy(sds);
    }
    PyFloat::Builtin(python_round_int(sds / (2.0 / 3.0)) * (2.0 / 3.0))
}

/// The SDS a line is drawn at and its centile, for a requested centile or,
/// if `is_sds`, SDS.
///
/// # Errors
///
/// [`ChartError::NonFinite`] where upstream's rounding of the SDS fails.
fn line_sds(
    value: f64,
    is_sds: bool,
    cole: bool,
    rounding: LineSds,
) -> Result<(PyFloat, f64), ChartError> {
    let (z, centile) = if is_sds {
        (PyFloat::Builtin(value), normal::cdf(value) * 100.0)
    } else if cole {
        (cole_sds_for_centile(value), value)
    } else {
        (PyFloat::NumPy(normal::ppf(value / 100.0)), value)
    };
    // Upstream fails where it rounds 100 times the SDS to an integer; the
    // Trisomy 21 builder rounds the SDS itself, which fails only at infinity.
    let overflows = match rounding {
        LineSds::Rounded => !z.value().is_finite(),
        LineSds::Hundredths => !(z.value() * 100.0).is_finite(),
    };
    if overflows {
        return Err(ChartError::NonFinite(value));
    }
    Ok((z, centile))
}

#[allow(clippy::too_many_arguments, reason = "mirrors the upstream loop state")]
fn line(
    data: &dyn ReferenceData,
    definition: &Definition,
    method: MeasurementMethod,
    sex: Sex,
    segment: &Segment,
    value: f64,
    is_sds: bool,
    cole: bool,
    tables_read: &mut Vec<TableId>,
) -> Result<ChartLine, ChartError> {
    let (z, centile) = line_sds(value, is_sds, cole, definition.chart.line_sds)?;
    let label = if is_sds { z.round(3).value() } else { centile };
    let requested = z.round(4);

    let grid = (segment.ages)(method);
    let mut points = Vec::with_capacity(grid.len());
    for &age in grid {
        match measurement_from_sds(
            data,
            definition.reference,
            requested,
            method,
            sex,
            age,
            (segment.youngest_at)(age),
        ) {
            Ok((y, table)) => {
                tables_read.push(table);
                points.push(ChartPoint {
                    label,
                    x: python_round(age, 4),
                    y: y.map(PyFloat::value),
                });
            }
            Err(CalcError::Data(error @ DataError::Unavailable { .. })) => return Err(error.into()),
            // Upstream skips any age where the calculation fails.
            Err(_) => {}
        }
    }
    // Upstream's Trisomy 21 builder rounds the SDS itself, as its kind rounds;
    // the others round 100 times it to an integer.
    let sds = match definition.chart.line_sds {
        LineSds::Rounded => z.round(2).value(),
        LineSds::Hundredths => python_round_int(z.value() * 100.0) / 100.0,
    };
    Ok(ChartLine {
        sds,
        centile,
        data: points,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cole_sds_are_two_thirds_apart() {
        let sds: Vec<f64> = CentileFormat::ColeNineCentiles
            .centiles()
            .iter()
            .map(|&c| cole_sds_for_centile(c).value())
            .collect();
        assert_eq!(sds[4], 0.0);
        for pair in sds.windows(2) {
            assert!((pair[1] - pair[0] - 2.0 / 3.0).abs() < 1e-12);
        }
    }

    #[test]
    fn centile_zero_or_hundred_has_no_finite_sds() {
        struct Nothing;
        impl ReferenceData for Nothing {
            fn table(
                &self,
                table: crate::data::TableId,
                _: MeasurementMethod,
                _: Sex,
            ) -> Result<&crate::table::LmsTable, DataError> {
                Err(DataError::Unavailable { table })
            }
        }
        for bad in [0.0, 100.0, 150.0] {
            let values = [bad];
            let result = chart(
                &Nothing,
                GrowthReference::UkWho,
                MeasurementMethod::Height,
                Sex::Male,
                CentileSpec::Custom {
                    values: &values,
                    is_sds: false,
                },
            );
            assert!(matches!(result, Err(ChartError::NonFinite(_))), "{bad}");
        }
    }

    #[test]
    fn every_named_collection_has_a_finite_sds_for_every_line() {
        // The server relies on this: it treats `NonFinite` from a named
        // collection as unreachable.
        let names = [
            "cole-nine-centiles",
            "three-percent-centiles",
            "five-percent-centiles",
            "eighty-five-percent-centiles",
        ];
        for name in names {
            let format = CentileFormat::from_name(name).unwrap();
            for &centile in format.centiles() {
                for cole in [false, true] {
                    for rounding in [LineSds::Hundredths, LineSds::Rounded] {
                        let result = line_sds(centile, false, cole, rounding);
                        assert!(result.is_ok(), "{name}, {centile}, {cole}, {rounding:?}");
                    }
                }
            }
        }
    }

    #[test]
    fn cole_median_sds_is_a_numpy_float_and_the_others_python_floats() {
        assert_eq!(cole_sds_for_centile(50.0), PyFloat::NumPy(0.0));
        assert!(matches!(cole_sds_for_centile(91.0), PyFloat::Builtin(_)));
    }

    #[cfg(feature = "embedded-data")]
    #[test]
    fn a_line_just_below_the_median_has_sds_positive_zero() {
        // Upstream's round(z * 100) / 100 passes through an int, so a line at
        // z = -0.0025 has "sds": 0.0, not -0.0.
        let store = crate::store::ReferenceStore::embedded().unwrap();
        let values = [49.9];
        let chart = chart(
            &store,
            GrowthReference::UkWho,
            MeasurementMethod::Weight,
            Sex::Female,
            CentileSpec::Custom {
                values: &values,
                is_sds: false,
            },
        )
        .unwrap();
        assert_eq!(chart.segments[0].lines[0].sds.to_bits(), 0.0_f64.to_bits());
    }
}
