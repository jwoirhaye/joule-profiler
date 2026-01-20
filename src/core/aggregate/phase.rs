use std::ops::AddAssign;

use crate::core::{aggregate::metric::Metrics, source::types::RawPhase};

#[derive(Default, Debug)]

pub struct SensorPhase {
    pub metrics: Metrics,
}

impl SensorPhase {
    pub fn new(metrics: Metrics) -> Self {
        Self { metrics }
    }
}

impl AddAssign for SensorPhase {
    fn add_assign(&mut self, rhs: Self) {
        self.metrics.extend(rhs.metrics);
    }
}

impl<V> From<RawPhase<V>> for SensorPhase
where
    V: Into<Metrics>,
{
    fn from(phase: RawPhase<V>) -> Self {
        SensorPhase::new(phase.metrics.into())
    }
}
