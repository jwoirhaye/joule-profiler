//! Measurement units and SI prefixes.
//!
//! This module defines basic units, SI prefixes, and their composition
//! into metric units used throughout the profiler.

use serde::{Serialize, Serializer};
use std::fmt::Display;

use crate::JouleProfilerError;

/// SI prefixes used to scale metric units.
#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq)]
pub enum UnitPrefix {
    /// Nano prefix (10^-9).
    Nano,

    /// Micro prefix (10^-6).
    Micro,

    /// Milli prefix (10^-3).
    Milli,

    /// No prefix (base unit).
    None,

    /// Kilo prefix (10^3).
    Kilo,

    /// Mega prefix (10^6).
    Mega,

    /// Giga prefix (10^9).
    Giga,
}

impl Display for UnitPrefix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            UnitPrefix::Nano => "n",
            UnitPrefix::Micro => "µ",
            UnitPrefix::Milli => "m",
            UnitPrefix::None => "",
            UnitPrefix::Kilo => "k",
            UnitPrefix::Mega => "M",
            UnitPrefix::Giga => "G",
        })
    }
}

/// Base measurement units.
#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq)]
pub enum Unit {
    /// Energy unit.
    Joule,

    /// Power unit.
    Watt,

    /// Time unit.
    Second,

    /// Count.
    Count,

    /// Memory or data size.
    Byte,

    /// Percentage.
    Percent,
}

impl Display for Unit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Unit::Joule => "J",
            Unit::Watt => "W",
            Unit::Second => "s",
            Unit::Count => "count",
            Unit::Byte => "B",
            Unit::Percent => "%",
        })
    }
}

/// A metric unit composed of an SI prefix and a base unit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MetricUnit {
    /// SI prefix applied to the unit.
    pub prefix: UnitPrefix,

    /// Base measurement unit.
    pub unit: Unit,
}

impl Serialize for MetricUnit {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl Display for MetricUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.prefix, self.unit)
    }
}

impl TryFrom<&str> for MetricUnit {
    type Error = JouleProfilerError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        if s.is_empty() {
            return Err(JouleProfilerError::InvalidUnit(s.into()));
        }

        const PREFIXES: &[(&str, UnitPrefix)] = &[
            ("n", UnitPrefix::Nano),
            ("µ", UnitPrefix::Micro),
            ("u", UnitPrefix::Micro),
            ("m", UnitPrefix::Milli),
            ("k", UnitPrefix::Kilo),
            ("M", UnitPrefix::Mega),
            ("G", UnitPrefix::Giga),
        ];

        let (prefix, unit_str) = PREFIXES
            .iter()
            .find_map(|(p, prefix)| {
                if s.starts_with(p) {
                    Some((*prefix, &s[p.len()..]))
                } else {
                    None
                }
            })
            .unwrap_or((UnitPrefix::None, s));

        if unit_str.is_empty() {
            return Err(JouleProfilerError::InvalidUnit(s.into()));
        }

        let unit = match unit_str {
            "J" => Unit::Joule,
            "W" => Unit::Watt,
            "s" => Unit::Second,
            "count" => Unit::Count,
            "B" => Unit::Byte,
            "%" => Unit::Percent,
            _ => return Err(JouleProfilerError::InvalidUnit(s.into())),
        };

        if matches!(unit, Unit::Count) && prefix != UnitPrefix::None {
            return Err(JouleProfilerError::InvalidUnit(s.into()));
        }

        Ok(MetricUnit { prefix, unit })
    }
}
