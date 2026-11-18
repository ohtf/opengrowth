// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-FileCopyrightText: Royal College of Paediatrics and Child Health
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Canonical names for growth references, sexes, and measurement methods.
//!
//! The string forms are part of the public contract shared with the RCPCH
//! Digital Growth Charts API and must not change. See
//! `spec/ubiquitous-language.md`.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// A growth reference: the population data a measurement is compared against.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum GrowthReference {
    /// UK-WHO: UK90 preterm, WHO 2006 from 2 weeks to 4 years, UK90 from 4 to 20 years.
    #[serde(rename = "uk-who")]
    UkWho,
    /// Turner syndrome (girls only).
    #[serde(rename = "turners-syndrome")]
    TurnersSyndrome,
    /// UK Trisomy 21 (Down syndrome) reference.
    #[serde(rename = "trisomy-21")]
    Trisomy21,
    /// American Academy of Pediatrics Trisomy 21 reference.
    #[serde(rename = "trisomy-21-aap")]
    Trisomy21Aap,
    /// US Centers for Disease Control and Prevention reference.
    #[serde(rename = "cdc")]
    Cdc,
    /// World Health Organization 2006 standard and 2007 reference.
    #[serde(rename = "who")]
    Who,
}

impl GrowthReference {
    /// Every reference, in the order the upstream engine lists them.
    pub const ALL: [Self; 6] = [
        Self::UkWho,
        Self::Trisomy21,
        Self::Trisomy21Aap,
        Self::TurnersSyndrome,
        Self::Cdc,
        Self::Who,
    ];

    /// The canonical wire spelling, for example `"uk-who"`.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UkWho => "uk-who",
            Self::TurnersSyndrome => "turners-syndrome",
            Self::Trisomy21 => "trisomy-21",
            Self::Trisomy21Aap => "trisomy-21-aap",
            Self::Cdc => "cdc",
            Self::Who => "who",
        }
    }
}

/// Biological sex as used to select reference data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Sex {
    /// Male.
    Male,
    /// Female.
    Female,
}

impl Sex {
    /// The canonical wire spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Male => "male",
            Self::Female => "female",
        }
    }
}

/// What was measured.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MeasurementMethod {
    /// Length (lying) or height (standing), in cm.
    #[serde(rename = "height")]
    Height,
    /// Weight, in kg.
    #[serde(rename = "weight")]
    Weight,
    /// Body mass index, in kg/m².
    #[serde(rename = "bmi")]
    Bmi,
    /// Occipitofrontal (head) circumference, in cm.
    #[serde(rename = "ofc")]
    Ofc,
}

impl MeasurementMethod {
    /// Every measurement method.
    pub const ALL: [Self; 4] = [Self::Height, Self::Weight, Self::Bmi, Self::Ofc];

    /// The canonical wire spelling, for example `"ofc"`.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Height => "height",
            Self::Weight => "weight",
            Self::Bmi => "bmi",
            Self::Ofc => "ofc",
        }
    }
}

/// A string did not name a known value.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("unknown {kind} `{value}`; expected one of: {expected}")]
pub struct ParseVocabularyError {
    kind: &'static str,
    value: String,
    expected: &'static str,
}

macro_rules! impl_text {
    ($ty:ty, $kind:literal, $expected:literal, [$($variant:ident),+]) => {
        impl fmt::Display for $ty {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(self.as_str())
            }
        }

        impl FromStr for $ty {
            type Err = ParseVocabularyError;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                $(
                    if value == <$ty>::$variant.as_str() {
                        return Ok(<$ty>::$variant);
                    }
                )+
                Err(ParseVocabularyError {
                    kind: $kind,
                    value: value.to_owned(),
                    expected: $expected,
                })
            }
        }
    };
}

impl_text!(
    GrowthReference,
    "growth reference",
    "uk-who, turners-syndrome, trisomy-21, trisomy-21-aap, cdc, who",
    [UkWho, TurnersSyndrome, Trisomy21, Trisomy21Aap, Cdc, Who]
);
impl_text!(Sex, "sex", "male, female", [Male, Female]);
impl_text!(
    MeasurementMethod,
    "measurement method",
    "height, weight, bmi, ofc",
    [Height, Weight, Bmi, Ofc]
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strings_round_trip_through_display_from_str_and_serde() {
        for reference in GrowthReference::ALL {
            let text = reference.to_string();
            assert_eq!(text.parse::<GrowthReference>().unwrap(), reference);
            let json = serde_json::to_string(&reference).unwrap();
            assert_eq!(json, format!("\"{text}\""));
        }
        for method in MeasurementMethod::ALL {
            assert_eq!(
                method.to_string().parse::<MeasurementMethod>().unwrap(),
                method
            );
        }
        assert_eq!("female".parse::<Sex>().unwrap(), Sex::Female);
    }

    #[test]
    fn canonical_spellings_are_exact() {
        // `turners-syndrome`, never `turner`: provenance invariant 4.
        assert_eq!(
            GrowthReference::TurnersSyndrome.as_str(),
            "turners-syndrome"
        );
        assert!("turner".parse::<GrowthReference>().is_err());
        assert!("Male".parse::<Sex>().is_err());
    }
}
