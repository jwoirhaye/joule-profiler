use std::ops::AddAssign;

use crate::core::{aggregate::metric::Metrics, source::types::RawPhase};

/// Aggregated metrics for a sensor phase
#[derive(Default, Debug)]
pub struct SensorPhase {
    /// Metrics associated with this phase
    pub metrics: Metrics,
}

impl SensorPhase {
    /// Creates a new sensor phase
    pub fn new(metrics: Metrics) -> Self {
        Self { metrics }
    }
}

impl AddAssign for SensorPhase {
    /// Merges metrics from another phase
    fn add_assign(&mut self, rhs: Self) {
        self.metrics.extend(rhs.metrics);
    }
}

impl<V> From<RawPhase<V>> for SensorPhase
where
    V: Into<Metrics>,
{
    /// Converts a raw phase into a sensor phase
    fn from(phase: RawPhase<V>) -> Self {
        SensorPhase::new(phase.metrics.into())
    }
}
