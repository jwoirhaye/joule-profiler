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
    pub value: u64,

    /// The unit of measurement.
    pub unit: MetricUnit,

    /// The source providing this metric (e.g. rapl).
    pub source: String,
}

/// A collection of metrics.
pub type Metrics = Vec<Metric>;
