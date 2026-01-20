use serde::Serialize;

/// Represents a measurable sensor
#[derive(Debug, Serialize)]
pub struct Sensor {
    /// Sensor name
    pub name: String,

    /// Unit of the sensor value
    pub unit: String,

    /// Source or origin of the sensor
    pub source: String,
}

impl Sensor {
    /// Create a new Sensor
    pub fn new(name: String, unit: String, source: String) -> Self {
        Self { name, unit, source }
    }
}

/// Collection of sensors
pub type Sensors = Vec<Sensor>;