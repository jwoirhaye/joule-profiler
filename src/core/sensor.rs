use std::ops::Add;

use serde::Serialize;

use crate::core::source::SensorIteration;

#[derive(Serialize)]
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

#[derive(Debug)]
pub struct SensorResult {
    pub iterations: Vec<SensorIteration>,
}

impl Add for SensorResult {
    type Output = SensorResult;

    fn add(self, rhs: Self) -> Self::Output {
        let iterations = self
            .iterations
            .into_iter()
            .zip(rhs.iterations)
            .map(|(self_iter, rhs_iter)| self_iter + rhs_iter)
            .collect();
        Self::Output { iterations }
    }
}

impl SensorResult {
    pub fn new(iterations: Vec<SensorIteration>) -> Self {
        Self { iterations }
    }

    pub fn merge(results: Vec<Self>) -> Option<SensorResult> {
        results.into_iter().reduce(|acc, result| acc + result)
    }
}
