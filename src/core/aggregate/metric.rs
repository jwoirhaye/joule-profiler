use serde::Serialize;

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
/// use joule_profiler::metrics::Metric;
///
/// let energy = Metric::new(
///     "energy_pkg".to_string(),
///     123456,
///     "µJ".to_string(),
///     "rapl".to_string(),
/// );
/// println!("Metric {}: {} {}", energy.name, energy.value, energy.unit);
/// ```
#[derive(Debug, Serialize, Clone)]
pub struct Metric {
    pub name: String,

    pub value: u64,

    pub unit: String,

    pub source: String,
}

impl Metric {
    /// Create a new metric with the given name, value, unit, and source.
    pub fn new(name: String, value: u64, unit: String, source: String) -> Self {
        Metric {
            name,
            value,
            unit,
            source,
        }
    }
}

/// A collection of metrics.
pub type Metrics = Vec<Metric>;
