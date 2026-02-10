use serde::Serialize;

use crate::unit::MetricUnit;

/// Represents a single measurable metric collected from a source.
///
/// A metric consists of a name, a numeric value, a unit of measurement,
/// and the source from which it was obtained.
///
/// # Fields
///
/// - `name` (`String`): The name of the metric, e.g., `"energy_pkg"`.
/// - `value` (`u64`): The numeric value of the metric.
/// - `unit` (`String`): The unit of measurement, e.g., `"µJ"` or `"W"`.
/// - `source` (`String`): The name of the source providing this metric,
///   e.g., `"rapl"` or `"powercap"`.
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
/// println!("Metric {}: {} {}", energy.name, energy.value, energy.unit);
/// ```
#[derive(Debug, Serialize, Clone)]
pub struct Metric {
    pub name: String,

    pub value: u64,

    pub unit: MetricUnit,

    pub source: String,
}

/// A collection of metrics.
pub type Metrics = Vec<Metric>;
