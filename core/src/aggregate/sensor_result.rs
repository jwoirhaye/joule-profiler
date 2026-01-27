use crate::aggregate::iteration::SensorIteration;
use std::ops::Add;

/// Aggregated result of multiple sensor iterations
#[derive(Debug)]
pub struct SensorResult {
    /// Iterations collected from all metric sources
    pub iterations: Vec<SensorIteration>,
}

impl SensorResult {
    /// Merge multiple SensorResults into one, returns None if empty
    pub fn merge(results: Vec<Self>) -> Option<SensorResult> {
        results.into_iter().reduce(|acc, result| acc + result)
    }
}

impl Add for SensorResult {
    type Output = SensorResult;

    /// Combine two sensor results by adding corresponding iterations
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
