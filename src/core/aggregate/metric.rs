use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct Metric {
    pub name: String,
    pub value: u64,
    pub unit: String,
    pub source: String,
}

impl Metric {
    pub fn new(name: String, value: u64, unit: String, source: String) -> Self {
        Metric {
            name,
            value,
            unit,
            source,
        }
    }
}

pub type Metrics = Vec<Metric>;
