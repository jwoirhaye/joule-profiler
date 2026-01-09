use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Metric {
    pub name: String,
    pub value: u64,
    pub source: String,
    pub unit: String,
}

impl Metric {
    pub fn new(name: String, value: u64, source: String, unit: String) -> Self {
        Metric {
            name,
            value,
            source,
            unit,
        }
    }
}

pub type Metrics = Vec<Metric>;

#[derive(Debug, Clone)]
pub struct Snapshot {
    pub metrics: Metrics,
    pub timestamp_us: u128,
}