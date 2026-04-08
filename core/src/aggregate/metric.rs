use std::fmt::Display;

use serde::Serialize;

use crate::unit::MetricUnit;

/// Represents a single measurable metric collected from a source.
///
/// # Examples
///
/// ```
/// use joule_profiler_core::{types::Metric, unit::{MetricUnit, Unit, UnitPrefix}};
///
/// let energy = Metric {
///     name: "energy_pkg".to_string(),
///     value: 123456,
///     unit: MetricUnit { unit: Unit::Joule, prefix: UnitPrefix::Micro },
///     source: "rapl".to_string(),
/// };
/// ```
#[derive(Debug, Serialize, Clone)]
pub struct Metric {
    /// The metric name, (e.g. `energy_pkg`).
    pub name: String,

    /// The numeric value of the metric.
    pub value: MetricValue,

    /// The unit of measurement.
    pub unit: MetricUnit,

    /// The source providing this metric (e.g. rapl).
    pub source: String,
}

impl Metric {
    pub fn new<T>(name: String, value: T, unit: MetricUnit, source: String) -> Self
    where
        T: Into<MetricValue>,
    {
        Metric {
            name,
            value: value.into(),
            unit,
            source,
        }
    }
}

/// A collection of metrics.
pub type Metrics = Vec<Metric>;

/// Enum representing the value of a metric,
/// with this enum, a metric can be a signed or
/// unsigned integer or a float.
#[derive(Debug, Serialize, Clone, Copy, PartialEq)]
pub enum MetricValue {
    UnsignedInteger(u64),
    SignedInteger(i64),
    Float(f64),
}

impl From<u64> for MetricValue {
    fn from(v: u64) -> Self {
        Self::UnsignedInteger(v)
    }
}
impl From<i64> for MetricValue {
    fn from(v: i64) -> Self {
        Self::SignedInteger(v)
    }
}
impl From<f64> for MetricValue {
    fn from(v: f64) -> Self {
        Self::Float(v)
    }
}

impl Display for MetricValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsignedInteger(v) => v.fmt(f),
            Self::SignedInteger(v) => v.fmt(f),
            Self::Float(v) => v.fmt(f),
        }
    }
}
