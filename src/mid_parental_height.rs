// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-FileCopyrightText: Royal College of Paediatrics and Child Health
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Mid-parental height: a child's expected adult height SDS from the heights
//! of the biological parents.
//!
//! Ported from `rcpchgrowth.mid_parental_height`. Each parent's height is
//! converted to an SDS at adult age, and the mid-parental SDS is half their
//! mean, `(maternal + paternal) / 4`, which applies the regression to the
//! mean of 0.5 (Wright CM, Cheetham TD. The strengths and limitations of
//! parental heights as a predictor of attained height. *Arch Dis Child*
//! 1999;81:257-260). Ninety per cent of children were within 1.4 SDS of it.

use crate::vocabulary::GrowthReference;

/// Width of the expected range either side of the mid-parental SDS.
pub const EXPECTED_RANGE_SDS: f64 = 1.4;

/// The age at which parents' heights are compared: 19 years for WHO, whose
/// reference ends there, and 20 otherwise (each reference's
/// [`Definition`](crate::references::Definition)).
#[must_use]
pub fn adult_age(reference: GrowthReference) -> f64 {
    crate::references::definition(reference).adult_age
}

/// The mid-parental SDS from the parents' SDS (upstream `mid_parental_height_z`).
#[must_use]
pub fn mid_parental_sds(maternal_sds: f64, paternal_sds: f64) -> f64 {
    (maternal_sds + paternal_sds) / 4.0
}

/// The range around the mid-parental SDS within which about 90% of children
/// fall (upstream `lower_and_upper_limits_of_expected_height_z`).
#[must_use]
pub fn expected_range(mid_parental_sds: f64) -> (f64, f64) {
    (
        mid_parental_sds - EXPECTED_RANGE_SDS,
        mid_parental_sds + EXPECTED_RANGE_SDS,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_mid_parental_sds_is_half_the_parents_mean() {
        assert_eq!(mid_parental_sds(1.0, 2.0), 0.75);
        assert_eq!(expected_range(0.5), (0.5 - 1.4, 0.5 + 1.4));
        assert_eq!(adult_age(GrowthReference::Who), 19.0);
        assert_eq!(adult_age(GrowthReference::UkWho), 20.0);
    }
}
