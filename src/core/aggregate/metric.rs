use serde::Serialize;

/// A single reported metric
#[derive(Debug, Serialize, Clone)]
pub struct Metric {
    /// Metric name
    pub name: String,

    /// Metric value
    pub value: u64,

    /// Unit of the metric value
    pub unit: String,

    /// Metric source
    pub source: String,
}

impl Metric {
    /// Creates a new metric
    pub fn new(name: String, value: u64, unit: String, source: String) -> Self {
        Metric {
            name,
            value,
            unit,
            source,
        }
    }
}

/// Collection of metrics
pub type Metrics = Vec<Metric>;
