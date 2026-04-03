use crate::aggregate::Metrics;
use crate::source::types::RawPhase;
use std::ops::{Add, AddAssign};

/// Aggregated metrics for a sensor phase.
#[derive(Default, Debug)]
pub struct SensorPhase {
    /// Metrics associated with this phase.
    pub metrics: Metrics,
}

impl AddAssign for SensorPhase {
    /// Merges metrics from another phase.
    fn add_assign(&mut self, rhs: Self) {
        self.metrics.extend(rhs.metrics);
    }
}

impl Add for SensorPhase {
    type Output = SensorPhase;

    fn add(mut self, rhs: Self) -> Self::Output {
        self += rhs;
        self
    }
}

impl<V> From<RawPhase<V>> for SensorPhase
where
    V: Into<Metrics>,
{
    fn from(phase: RawPhase<V>) -> Self {
        SensorPhase {
            metrics: phase.metrics.into(),
        }
    }
}
