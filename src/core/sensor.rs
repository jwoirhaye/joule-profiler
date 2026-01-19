use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Sensor {
    pub name: String,
    pub unit: String,
    pub source: String,
}

impl Sensor {
    pub fn new(name: String, unit: String, source: String) -> Self {
        Self { name, unit, source }
    }
}

pub type Sensors = Vec<Sensor>;
