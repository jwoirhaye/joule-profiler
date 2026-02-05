//! Measurement units and SI prefixes.
//!
//! This module defines basic units, SI prefixes, and their composition
//! into metric units used throughout the profiler.

use std::fmt::Display;

use serde::Serialize;

/// SI prefixes used to scale metric units.
#[derive(Debug, Serialize, Clone, Copy)]
pub enum MetricPrefix {
    /// Nano prefix (`n`, 10⁻⁹).
    Nano,
    /// Micro prefix (`µ`, 10⁻⁶).
    Micro,
    /// Milli prefix (`m`, 10⁻³).
    Milli,
    /// No prefix (base unit).
    None,
}

impl Display for MetricPrefix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            MetricPrefix::Nano => "n",
            MetricPrefix::Micro => "µ",
            MetricPrefix::Milli => "m",
            MetricPrefix::None => "",
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
}

impl Display for Unit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Unit::Joule => "J",
            Unit::Watt => "W",
            Unit::Second => "s",
        })
    }
}

/// A metric unit composed of an SI prefix and a base unit.
#[derive(Debug, Serialize, Clone, Copy)]
pub struct MetricUnit {
    /// SI prefix applied to the unit.
    pub prefix: MetricPrefix,
    /// Base measurement unit.
    pub unit: Unit,
}

impl Display for MetricUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.prefix, self.unit)
    }
}
