use std::pin::Pin;

use crate::core::source::{MetricSource, error::MetricSourceError, result::SensorResult};

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

#[derive(Debug, Default, Clone)]

pub struct RawPhase<V> {
    pub metrics: V,
}

impl<V> RawPhase<V> {
    pub fn new(metrics: V) -> Self {
        Self { metrics }
    }
}
