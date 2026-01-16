use serde::Serialize;

#[derive(Serialize, Clone, Debug)]
pub struct Metric {
    pub name: String,
    pub value: u64,
    pub unit: String,
    pub source: String,
}

pub type Metrics = Vec<Metric>;
