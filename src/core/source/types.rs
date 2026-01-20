use std::{
    ops::{Add, AddAssign},
    pin::Pin,
};

use crate::core::{
    metric::Metrics,
    source::{MetricSource, error::MetricSourceError, result::SensorResult},
};

#[derive(Default, Debug)]
pub struct SensorIteration {
    pub phases: Vec<SensorPhase>,
    pub measure_delta: u64,
    pub measure_count: u64,
}

impl AddAssign for SensorPhase {
    fn add_assign(&mut self, rhs: Self) {
        self.metrics.extend(rhs.metrics);
    }
}

impl AddAssign for SensorIteration {
    fn add_assign(&mut self, rhs: Self) {
        self.phases
            .iter_mut()
            .zip(rhs.phases)
            .for_each(|(self_phase, rhs_phase)| *self_phase += rhs_phase);
    }
}

impl Add for SensorIteration {
    type Output = SensorIteration;

    fn add(mut self, rhs: Self) -> Self::Output {
        self += rhs;
        self
    }
}

impl SensorIteration {
    pub fn new(phases: Vec<SensorPhase>, measure_delta: u64, measure_count: u64) -> Self {
        Self {
            phases,
            measure_delta,
            measure_count,
        }
    }
}

#[derive(Default, Debug)]

pub struct SensorPhase {
    pub metrics: Metrics,
}

#[derive(Debug, Clone, Copy)]
pub enum SourceEvent {
    Measure,
    NewPhase,
    NewIteration,
    StartPolling,
    StopPolling,
    JoinWorker,
}

pub type MetricSourceWorkerFuture = Pin<
    Box<
        dyn Future<Output = Result<(SensorResult, Box<dyn MetricSource>), MetricSourceError>>
            + Send,
    >,
>;
