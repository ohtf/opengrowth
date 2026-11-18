// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-FileCopyrightText: Royal College of Paediatrics and Child Health
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Advisory text: comments on age correction and centile bands.
//!
//! Ported from `rcpchgrowth.age_advice_strings` and `rcpchgrowth.centile_bands`.
//! The strings reach clinicians and families through the API and chart
//! component, so they are reproduced character for character, including
//! "Born Term." in one place and "Born at term." in another.

use crate::dates::Gestation;
use crate::normal;
use crate::pyfmt::python_repr;
use crate::vocabulary::{GrowthReference, MeasurementMethod};

/// Lay and clinician comments on chronological and corrected age.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgeComments {
    /// Lay comment on the corrected age.
    pub lay_corrected: String,
    /// Lay comment on the chronological age.
    pub lay_chronological: String,
    /// Clinician comment on the corrected age.
    pub clinician_corrected: String,
    /// Clinician comment on the chronological age.
    pub clinician_chronological: String,
}

/// The upstream `comment_prematurity_correction`.
///
/// "No correction" is decided by exact equality of the two decimal ages,
/// as upstream: UK-WHO corrects every gestation except exactly 40+0.
#[must_use]
pub fn prematurity_comments(
    chronological_age: f64,
    corrected_age: f64,
    gestation: Gestation,
    reference: GrowthReference,
) -> AgeComments {
    let (weeks, days) = (gestation.weeks, gestation.days);
    let all =
        |lay_corr: String, lay_chron: String, clin_corr: String, clin_chron: String| AgeComments {
            lay_corrected: lay_corr,
            lay_chronological: lay_chron,
            clinician_corrected: clin_corr,
            clinician_chronological: clin_chron,
        };

    if chronological_age == corrected_age {
        let no_correction_reference =
            matches!(reference, GrowthReference::Cdc | GrowthReference::Who);
        if weeks < 42 && no_correction_reference {
            if weeks < 37 {
                let lay = format!(
                    "Your child was born at {weeks}+{days} weeks gestation. No correction is made for this beyond 2 years of age."
                );
                let clinician = format!(
                    "Born preterm at {weeks}+{days} weeks gestation. No correction is made for this beyond 2 years of age."
                );
                return all(lay.clone(), lay, clinician.clone(), clinician);
            }
            if weeks == 40 && days == 0 {
                let lay = "Your child was born at term. No correction for gestation has been made."
                    .to_owned();
                let clinician =
                    "Born at term. No correction for gestation has been made.".to_owned();
                return all(lay.clone(), lay, clinician.clone(), clinician);
            }
            let lay = format!(
                "Your child was born at {weeks}+{days} weeks gestation. This is considered term and no correction for gestation has been made."
            );
            let clinician = format!(
                "Born at term ({weeks}+{days} weeks gestation). No correction for gestation has been made."
            );
            return all(lay.clone(), lay, clinician.clone(), clinician);
        }
        return all(
            "Your child was born on their due date.".to_owned(),
            "Your child was born on their due date.".to_owned(),
            "Born at term. No correction has been made for gestation.".to_owned(),
            "Born Term. No correction has been made for gestation.".to_owned(),
        );
    }
    if chronological_age.is_nan() || corrected_age.is_nan() {
        let text = "It has not been possible to calculate age this time.".to_owned();
        return all(text.clone(), text.clone(), text.clone(), text);
    }
    all(
        format!(
            "Because your child was born at {weeks}+{days} weeks gestation an adjustment has been made to take this into account."
        ),
        "This is your child's age without taking into account their gestation at birth.".to_owned(),
        "Correction for gestational age has been made.".to_owned(),
        "No correction has been made for gestational age.".to_owned(),
    )
}

/// A named collection of centile lines.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CentileFormat {
    /// 0.4, 2, 9, 25, 50, 75, 91, 98, 99.6: two-thirds of an SD apart (UK-WHO).
    ColeNineCentiles,
    /// 3, 5, 10, 25, 50, 75, 90, 95, 97 (CDC).
    ThreePercentCentiles,
    /// 5, 10, 25, 50, 75, 90, 95.
    FivePercentCentiles,
    /// 5 to 99.99, for CDC extended BMI.
    EightyFivePercentCentiles,
}

impl CentileFormat {
    /// The collection for an upstream format name, if opengrowth has it.
    /// `extended-who-centiles` is accepted upstream but has no data (`Q7`).
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "cole-nine-centiles" => Some(Self::ColeNineCentiles),
            "three-percent-centiles" => Some(Self::ThreePercentCentiles),
            "five-percent-centiles" => Some(Self::FivePercentCentiles),
            "eighty-five-percent-centiles" => Some(Self::EightyFivePercentCentiles),
            _ => None,
        }
    }

    /// The centiles in the collection, as upstream lists them.
    #[must_use]
    pub const fn centiles(self) -> &'static [f64] {
        match self {
            Self::ColeNineCentiles => &[0.4, 2.0, 9.0, 25.0, 50.0, 75.0, 91.0, 98.0, 99.6],
            Self::ThreePercentCentiles => &[3.0, 5.0, 10.0, 25.0, 50.0, 75.0, 90.0, 95.0, 97.0],
            Self::FivePercentCentiles => &[5.0, 10.0, 25.0, 50.0, 75.0, 90.0, 95.0],
            Self::EightyFivePercentCentiles => &[
                5.0, 10.0, 25.0, 50.0, 75.0, 85.0, 90.0, 95.0, 98.0, 99.0, 99.9, 99.99,
            ],
        }
    }
}

/// Advisory SDS limits beyond which a measurement is "well outside the normal range".
const ADVISORY_SDS: f64 = 4.0;
/// Half-width of the "on or near" band around a centile line, in SDS:
/// a quarter of the two-thirds-SD spacing, as upstream (0.25 * 0.666).
const QUARTER_DISTANCE: f64 = 0.25 * 0.666;

/// The centile band sentence for an SDS (the upstream `centile_band_for_centile`).
///
/// `None` where upstream returns `None`: an SDS exactly on the upper edge
/// of the highest "on or near" band, or NaN. Neither occurs in practice.
#[must_use]
pub fn centile_band(sds: f64, method: MeasurementMethod, format: CentileFormat) -> Option<String> {
    let centiles = format.centiles();
    // Upstream uses the same +/-4 advisory limits for BMI and other measurements.
    let name = match method {
        MeasurementMethod::Bmi => "body mass index",
        MeasurementMethod::Ofc => "head circumference",
        MeasurementMethod::Height => "height",
        MeasurementMethod::Weight => "weight",
    };
    let edges: Vec<f64> = centiles
        .iter()
        .flat_map(|&c| {
            let line = normal::ppf(c / 100.0);
            [line - QUARTER_DISTANCE, line + QUARTER_DISTANCE]
        })
        .collect();

    // Not `!range.contains()`: that would treat NaN as out of range, which upstream does not.
    #[allow(
        clippy::manual_range_contains,
        reason = "NaN must fall through, as upstream"
    )]
    let outside = sds < -ADVISORY_SDS || sds > ADVISORY_SDS;
    if outside {
        return Some(format!(
            "This {name} measurement is well outside the normal range. Please check its accuracy."
        ));
    }
    if sds <= edges[0] {
        return Some(format!(
            "This {name} measurement is below the normal range."
        ));
    }
    if sds > edges[edges.len() - 1] {
        return Some(format!(
            "This {name} measurement is above the normal range."
        ));
    }
    for (band, pair) in edges.windows(2).enumerate() {
        if pair[0] <= sds && sds < pair[1] {
            return Some(if band % 2 == 0 {
                let centile = ordinal(centiles[band / 2]);
                format!("This {name} measurement is on or near the {centile} centile.")
            } else {
                let lower = ordinal(centiles[(band - 1) / 2]);
                let upper = ordinal(centiles[band.div_ceil(2)]);
                format!("This {name} measurement is between the {lower} and {upper} centiles.")
            });
        }
    }
    None
}

/// A centile as an ordinal, as upstream `return_suffix` writes it:
/// "2nd", "91st", "11th", "0.4th", "99.6th".
#[must_use]
pub fn ordinal(centile: f64) -> String {
    if centile <= 0.0 {
        return "below lowest centile.;".to_owned();
    }
    if centile >= 100.0 {
        return "above highest centile.".to_owned();
    }
    let whole = centile.fract() == 0.0;
    #[allow(clippy::cast_possible_truncation, reason = "whole numbers below 100")]
    let text = if whole {
        (centile as i64).to_string()
    } else {
        python_repr(centile)
    };
    let mut suffix = match text.as_bytes()[text.len() - 1] {
        b'1' => "st",
        b'2' => "nd",
        b'3' => "rd",
        _ => "th",
    };
    if whole && text.len() > 1 {
        let last_two: i64 = text[text.len() - 2..].parse().expect("digits");
        if (11..=13).contains(&last_two) {
            suffix = "th";
        }
    }
    format!("{text}{suffix}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordinals_match_upstream() {
        let cases = [
            (0.4, "0.4th"),
            (2.0, "2nd"),
            (9.0, "9th"),
            (91.0, "91st"),
            (99.6, "99.6th"),
            (3.0, "3rd"),
            (11.0, "11th"),
            (12.0, "12th"),
            (13.0, "13th"),
            (99.99, "99.99th"),
            (85.0, "85th"),
        ];
        for (c, expected) in cases {
            assert_eq!(ordinal(c), expected);
        }
    }

    #[test]
    fn bands_cover_the_upstream_sentences() {
        let band = |sds| {
            centile_band(
                sds,
                MeasurementMethod::Height,
                CentileFormat::ColeNineCentiles,
            )
            .unwrap()
        };
        assert_eq!(
            band(0.0),
            "This height measurement is on or near the 50th centile."
        );
        assert_eq!(
            band(-2.4),
            "This height measurement is between the 0.4th and 2nd centiles."
        );
        assert_eq!(
            band(-3.5),
            "This height measurement is below the normal range."
        );
        assert_eq!(
            band(3.5),
            "This height measurement is above the normal range."
        );
        assert_eq!(
            band(-8.1),
            "This height measurement is well outside the normal range. Please check its accuracy."
        );
        assert_eq!(
            centile_band(0.0, MeasurementMethod::Ofc, CentileFormat::ColeNineCentiles).unwrap(),
            "This head circumference measurement is on or near the 50th centile."
        );
    }
}
