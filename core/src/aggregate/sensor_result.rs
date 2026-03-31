use crate::aggregate::iteration::SensorIteration;
use std::ops::Add;

/// Aggregated result of multiple sensor iterations.
#[derive(Debug)]
pub struct SensorResult {
    /// Iterations collected from all metric sources.
    pub iterations: Vec<SensorIteration>,
}

impl SensorResult {
    /// Merge multiple sensor results into one, returns None if empty.
    pub fn merge(results: Vec<Self>) -> Option<SensorResult> {
        if results.is_empty() || results.iter().any(|r| !r.is_valid()) {
            return None;
        }
        results.into_iter().reduce(|acc, result| acc + result)
    }

    /// Test if result is valid by checking if all iterations contains phases.
    fn is_valid(&self) -> bool {
        !self.iterations.is_empty() && self.iterations.iter().all(|i| !i.phases.is_empty())
    }
}

impl Add for SensorResult {
    type Output = SensorResult;

    /// Combine two sensor results by adding corresponding iterations.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aggregate::{iteration::SensorIteration, phase::SensorPhase};
    use crate::types::Metric;
    use crate::unit::{MetricUnit, Unit, UnitPrefix};

    fn metric(value: u64) -> Metric {
        Metric {
            name: "energy_pkg".to_string(),
            value,
            unit: MetricUnit {
                unit: Unit::Joule,
                prefix: UnitPrefix::Micro,
            },
            source: "rapl".to_string(),
        }
    }

    fn phase(metrics: Vec<Metric>) -> SensorPhase {
        SensorPhase { metrics }
    }

    fn iteration(phases: Vec<SensorPhase>) -> SensorIteration {
        SensorIteration { phases }
    }

    fn result(iterations: Vec<SensorIteration>) -> SensorResult {
        SensorResult { iterations }
    }

    #[test]
    fn merge_empty_vec_returns_none() {
        assert!(SensorResult::merge(vec![]).is_none());
    }

    #[test]
    fn merge_with_empty_iteration_returns_none() {
        let r = result(vec![iteration(vec![])]);
        assert!(SensorResult::merge(vec![r]).is_none());
    }

    #[test]
    fn merge_single_result_returns_it() {
        let r = result(vec![iteration(vec![phase(vec![metric(100)])])]);
        let merged = SensorResult::merge(vec![r]).unwrap();
        assert_eq!(merged.iterations.len(), 1);
        assert_eq!(merged.iterations[0].phases[0].metrics[0].value, 100);
    }

    #[test]
    fn merge_multiple_results_accumulates_metrics() {
        let r1 = result(vec![iteration(vec![phase(vec![metric(100)])])]);
        let r2 = result(vec![iteration(vec![phase(vec![metric(200)])])]);
        let merged = SensorResult::merge(vec![r1, r2]).unwrap();

        assert_eq!(merged.iterations[0].phases[0].metrics.len(), 2);
        assert_eq!(merged.iterations[0].phases[0].metrics[0].value, 100);
        assert_eq!(merged.iterations[0].phases[0].metrics[1].value, 200);
    }
}
