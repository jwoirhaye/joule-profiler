//! Measurement units and SI prefixes.
//!
//! This module defines basic units, SI prefixes, and their composition
//! into metric units used throughout the profiler.

use serde::{Serialize, Serializer};
use std::fmt::Display;

/// SI prefixes used to scale metric units.
#[derive(Debug, Serialize, Clone, Copy)]
pub enum UnitPrefix {
    /// Nano prefix (`n`, 10^-9).
    Nano,
    /// Micro prefix (`µ`, 10^-6).
    Micro,
    /// Milli prefix (`m`, 10^-3).
    Milli,
    /// No prefix (base unit).
    None,
    /// Kilo prefix (`k`, 10^3).
    Kilo,
    /// Mega prefix (`M`, 10^6).
    Mega,
    /// Giga prefix (`G`, 10^9).
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
#[derive(Debug, Serialize, Clone, Copy)]
pub enum Unit {
    /// Energy unit (joule).
    Joule,
    /// Power unit (watt).
    Watt,
    /// Time unit (second).
    Second,
    /// Count (unitless event counts).
    Count,
    /// Memory or data size (byte).
    Byte,
    /// Percentage, ratio (0-100%).
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
#[derive(Debug, Clone, Copy)]
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
