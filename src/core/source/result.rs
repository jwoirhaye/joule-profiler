use std::ops::Add;

use crate::core::source::types::SensorIteration;

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
