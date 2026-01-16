use serde::Serialize;

#[derive(Serialize)]
pub struct Sensor {
    pub name: String,
    pub unit: String,
    pub source: String,
}
