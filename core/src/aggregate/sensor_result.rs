use crate::aggregate::phase::SensorPhase;
use std::ops::Add;

/// Aggregated results of sensors.
#[derive(Debug)]
pub struct SensorResult {
    /// Phases collected from all metric sources.
    pub phases: Vec<SensorPhase>,
}

impl SensorResult {
    /// Merge multiple sensor results into one.
    pub fn merge(results: Vec<Self>) -> Option<SensorResult> {
        if results.is_empty() || results.iter().any(|result| result.phases.is_empty()) {
            None
        } else {
            results.into_iter().reduce(|acc, result| acc + result)
        }
    }
}

impl Add for SensorResult {
    type Output = SensorResult;

    /// Combine two sensor results by adding corresponding phases.
    fn add(self, rhs: Self) -> Self::Output {
        let phases = self
            .phases
            .into_iter()
            .zip(rhs.phases)
            .map(|(self_iter, rhs_iter)| self_iter + rhs_iter)
            .collect();
        Self::Output { phases }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aggregate::phase::SensorPhase;
    use crate::types::{Metric, MetricValue};
    use crate::unit::{MetricUnit, Unit, UnitPrefix};

    fn metric(value: u64) -> Metric {
        let unit = MetricUnit {
            unit: Unit::Joule,
            prefix: UnitPrefix::Micro,
        };
        Metric::new("energy_pkg".to_string(), value, unit, "rapl".to_string())
    }

    fn phase(metrics: Vec<Metric>) -> SensorPhase {
        SensorPhase { metrics }
    }

    fn result(phases: Vec<SensorPhase>) -> SensorResult {
        SensorResult { phases }
    }

    #[test]
    fn merge_empty_vec_returns_none() {
        assert!(SensorResult::merge(vec![]).is_none());
    }

    #[test]
    fn merge_single_result_returns_it() {
        let r = result(vec![phase(vec![metric(100)])]);
        let merged = SensorResult::merge(vec![r]).unwrap();
        assert_eq!(merged.phases.len(), 1);
        assert_eq!(
            merged.phases[0].metrics[0].value,
            MetricValue::UnsignedInteger(100)
        );
    }

    #[test]
    fn merge_multiple_results_accumulates_metrics() {
        let r1 = result(vec![phase(vec![metric(100)])]);
        let r2 = result(vec![phase(vec![metric(200)])]);
        let merged = SensorResult::merge(vec![r1, r2]).unwrap();

        assert_eq!(merged.phases[0].metrics.len(), 2);
        assert_eq!(
            merged.phases[0].metrics[0].value,
            MetricValue::UnsignedInteger(100)
        );
        assert_eq!(
            merged.phases[0].metrics[1].value,
            MetricValue::UnsignedInteger(200)
        );
    }
}
