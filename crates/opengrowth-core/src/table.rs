// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-FileCopyrightText: Royal College of Paediatrics and Child Health
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Validated LMS tables and age lookup.
//!
//! Lookup reproduces `rcpchgrowth.global_functions.nearest_lowest_index` and
//! `fetch_lms`:
//!
//! 1. An age is an *exact match* for a table row when both round to the same
//!    value at four decimal places (Python `round`, ties to even). The row's
//!    values are used as they are.
//! 2. Otherwise the row immediately below the age is found. Cubic
//!    interpolation is used when there is one row below that and two rows
//!    from it upwards, the table has no sigma column, and the caller has not
//!    asked for linear interpolation. Otherwise linear interpolation is used.
//!
//! The upstream scan is linear; this one is a binary search over the same
//! ordering, which is equivalent because tables are validated as strictly
//! ascending. `tests::binary_search_matches_upstream_linear_scan` checks the
//! equivalence against a direct port of the upstream loop.

use crate::interpolation::{cubic_lagrange, linear};
use crate::lms::Lms;
use crate::round::python_round;

/// One row of an LMS table.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LmsPoint {
    /// Decimal age in years. Negative ages are weeks of gestation before 40 weeks.
    pub age: f64,
    /// Box-Cox power.
    pub l: f64,
    /// Median.
    pub m: f64,
    /// Coefficient of variation.
    pub s: f64,
    /// CDC extended-BMI sigma, present only in CDC BMI tables.
    pub sigma: Option<f64>,
}

/// How to interpolate between rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Interpolation {
    /// Cubic where four surrounding rows exist, linear at the table edges.
    CubicWherePossible,
    /// Always linear. The upstream engine forces this for WHO SDS.
    LinearOnly,
}

/// A table could not be built or could not answer a lookup.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum TableError {
    /// The table has no rows.
    #[error("the LMS table is empty")]
    Empty,
    /// A value is NaN or infinite.
    #[error("row {row} of the LMS table contains a non-finite value")]
    NonFinite {
        /// Zero-based row index.
        row: usize,
    },
    /// M or S is not positive, so the LMS transform is undefined.
    #[error("row {row} of the LMS table has a non-positive M or S")]
    NonPositive {
        /// Zero-based row index.
        row: usize,
    },
    /// Ages are not strictly ascending.
    #[error("row {row} of the LMS table is not in strictly ascending age order")]
    NotAscending {
        /// Zero-based row index.
        row: usize,
    },
    /// Some rows have a sigma value and others do not.
    #[error("the LMS table has sigma values on some rows but not others")]
    InconsistentSigma,
    /// The age lies outside the tabulated range, so there is nothing to
    /// interpolate between. Upstream this is an `IndexError` or a SciPy
    /// bounds `ValueError`; neither is reachable through UK-WHO age selection.
    #[error("age {age} is outside the LMS table range {first} to {last}")]
    AgeOutOfRange {
        /// Requested age.
        age: f64,
        /// First tabulated age.
        first: f64,
        /// Last tabulated age.
        last: f64,
    },
}

/// An LMS table for one reference segment, measurement method, and sex.
///
/// Construction validates the invariants lookup depends on, so a table that
/// exists is safe to query.
#[derive(Debug, Clone, PartialEq)]
pub struct LmsTable {
    points: Vec<LmsPoint>,
    /// `python_round(age, 4)` for each row, computed once.
    rounded_ages: Vec<f64>,
    has_sigma: bool,
}

impl LmsTable {
    /// Builds a table, checking that it is non-empty, finite, strictly
    /// ascending in age, has positive M and S, and has sigma on all rows or none.
    ///
    /// # Errors
    ///
    /// Returns the first invariant the rows violate.
    pub fn new(points: Vec<LmsPoint>) -> Result<Self, TableError> {
        let first = points.first().ok_or(TableError::Empty)?;
        let has_sigma = first.sigma.is_some();
        for (row, point) in points.iter().enumerate() {
            let finite = [point.age, point.l, point.m, point.s]
                .into_iter()
                .chain(point.sigma)
                .all(f64::is_finite);
            if !finite {
                return Err(TableError::NonFinite { row });
            }
            if point.m <= 0.0 || point.s <= 0.0 {
                return Err(TableError::NonPositive { row });
            }
            if point.sigma.is_some() != has_sigma {
                return Err(TableError::InconsistentSigma);
            }
            if row > 0 && points[row - 1].age >= point.age {
                return Err(TableError::NotAscending { row });
            }
        }
        let rounded_ages = points.iter().map(|p| python_round(p.age, 4)).collect();
        Ok(Self {
            points,
            rounded_ages,
            has_sigma,
        })
    }

    /// The rows, in ascending age order.
    #[must_use]
    pub fn points(&self) -> &[LmsPoint] {
        &self.points
    }

    /// First tabulated age.
    #[must_use]
    pub fn first_age(&self) -> f64 {
        self.points[0].age
    }

    /// Last tabulated age.
    #[must_use]
    pub fn last_age(&self) -> f64 {
        self.points[self.points.len() - 1].age
    }

    /// The upstream `nearest_lowest_index`: the first row whose age rounds to
    /// the same four-decimal value as `age`, or failing that the last row
    /// strictly below `age`, or row 0 if there is none.
    #[must_use]
    pub fn nearest_lowest_index(&self, age: f64) -> usize {
        let rounded = python_round(age, 4);
        // Rounded ages are non-decreasing, so equal values form one run and
        // the first of them is the partition point.
        let first_not_below = self.rounded_ages.partition_point(|&r| r < rounded);
        if self.rounded_ages.get(first_not_below) == Some(&rounded) {
            return first_not_below;
        }
        self.points
            .partition_point(|p| p.age < age)
            .saturating_sub(1)
    }

    /// LMS values at `age`, exact or interpolated (the upstream `fetch_lms`).
    ///
    /// # Errors
    ///
    /// [`TableError::AgeOutOfRange`] if `age` is not an exact match and lies
    /// outside the tabulated range.
    pub fn lms_at(&self, age: f64, interpolation: Interpolation) -> Result<Lms, TableError> {
        let index = self.nearest_lowest_index(age);
        let below = &self.points[index];

        if python_round(age, 4) == self.rounded_ages[index] {
            return Ok(Lms {
                l: below.l,
                m: below.m,
                s: below.s,
                sigma: below.sigma,
            });
        }

        let out_of_range = || TableError::AgeOutOfRange {
            age,
            first: self.first_age(),
            last: self.last_age(),
        };
        let above = self.points.get(index + 1).ok_or_else(out_of_range)?;
        if age < below.age {
            return Err(out_of_range());
        }

        let cubic = index >= 1
            && index + 2 < self.points.len()
            && !self.has_sigma
            && interpolation == Interpolation::CubicWherePossible;

        if cubic {
            let two_below = &self.points[index - 1];
            let two_above = &self.points[index + 2];
            let ages = [two_below.age, below.age, above.age, two_above.age];
            let pick = |f: fn(&LmsPoint) -> f64| {
                cubic_lagrange(age, ages, [f(two_below), f(below), f(above), f(two_above)])
            };
            return Ok(Lms {
                l: pick(|p| p.l),
                m: pick(|p| p.m),
                s: pick(|p| p.s),
                sigma: None,
            });
        }

        let pick = |lo: f64, hi: f64| linear(age, below.age, above.age, lo, hi);
        Ok(Lms {
            l: pick(below.l, above.l),
            m: pick(below.m, above.m),
            s: pick(below.s, above.s),
            sigma: below.sigma.zip(above.sigma).map(|(lo, hi)| pick(lo, hi)),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table(ages: &[f64]) -> LmsTable {
        LmsTable::new(
            ages.iter()
                .enumerate()
                .map(|(i, &age)| LmsPoint {
                    age,
                    l: 1.0 - 0.01 * i as f64,
                    m: 50.0 + i as f64,
                    s: 0.04 + 0.001 * i as f64,
                    sigma: None,
                })
                .collect(),
        )
        .unwrap()
    }

    /// Direct port of the upstream loop, kept as the oracle for the binary search.
    fn upstream_nearest_lowest_index(points: &[LmsPoint], age: f64) -> usize {
        let mut lowest_index = 0;
        for (num, element) in points.iter().enumerate() {
            if python_round(element.age, 4) == python_round(age, 4) {
                return num;
            } else if element.age < age {
                lowest_index = num;
            }
        }
        lowest_index
    }

    #[test]
    fn binary_search_matches_upstream_linear_scan() {
        // Daily ages as in the WHO tables, plus irregular and near-duplicate rows.
        let mut ages: Vec<f64> = (0..120).map(|d| f64::from(d) / 365.25).collect();
        ages.extend([0.5, 0.50004, 0.50006, 0.75, 1.0, 2.0, 4.0]);
        let table = table(&ages);
        let mut probe = -0.1;
        while probe < 4.2 {
            assert_eq!(
                table.nearest_lowest_index(probe),
                upstream_nearest_lowest_index(table.points(), probe),
                "age {probe}"
            );
            probe += 0.000_137;
        }
    }

    #[test]
    fn exact_match_within_four_decimals_uses_the_row() {
        let table = table(&[0.0, 1.0, 2.0, 3.0]);
        let lms = table
            .lms_at(1.000_04, Interpolation::CubicWherePossible)
            .unwrap();
        assert_eq!(lms.m, 51.0);
    }

    #[test]
    fn cubic_inside_and_linear_at_edges() {
        let table = table(&[0.0, 1.0, 2.0, 3.0, 4.0]);
        // index 1 with rows 0..=3 available: cubic. M is linear in age, so both agree.
        let inner = table
            .lms_at(1.5, Interpolation::CubicWherePossible)
            .unwrap();
        assert!((inner.m - 51.5).abs() < 1e-12);
        // index 0: linear only.
        let edge = table
            .lms_at(0.5, Interpolation::CubicWherePossible)
            .unwrap();
        assert_eq!(edge.m, 50.5);
    }

    #[test]
    fn out_of_range_is_an_error_not_a_panic() {
        let table = table(&[1.0, 2.0, 3.0]);
        assert!(matches!(
            table.lms_at(3.5, Interpolation::LinearOnly),
            Err(TableError::AgeOutOfRange { .. })
        ));
        assert!(matches!(
            table.lms_at(0.5, Interpolation::LinearOnly),
            Err(TableError::AgeOutOfRange { .. })
        ));
    }

    #[test]
    fn invalid_tables_are_rejected() {
        let point = |age, m| LmsPoint {
            age,
            l: 1.0,
            m,
            s: 0.1,
            sigma: None,
        };
        assert_eq!(LmsTable::new(vec![]), Err(TableError::Empty));
        assert_eq!(
            LmsTable::new(vec![point(1.0, 1.0), point(1.0, 1.0)]),
            Err(TableError::NotAscending { row: 1 })
        );
        assert_eq!(
            LmsTable::new(vec![point(1.0, 0.0)]),
            Err(TableError::NonPositive { row: 0 })
        );
        assert_eq!(
            LmsTable::new(vec![point(f64::NAN, 1.0)]),
            Err(TableError::NonFinite { row: 0 })
        );
    }
}
