use std::{fmt::Debug, pin::Pin, time::Duration};

use crate::core::{
    aggregate::{iteration::SensorIteration, metric::Metrics, sensor_result::SensorResult},
    source::{MetricSource, error::MetricSourceError},
};

/// Bounds of the type used in a metric reader
pub trait MetricReaderTypeBound: Debug + Send + Default + Into<Metrics> {}

impl<T> MetricReaderTypeBound for T where T: Debug + Default + Send + Into<Metrics> {}

pub trait MetricReaderErrorBound: std::error::Error + Send + Sync {}

impl<E> MetricReaderErrorBound for E where E: std::error::Error + Send + Sync {}

/// Future returned by a metric source worker
pub type MetricSourceFuture = Pin<
    Box<
        dyn Future<Output = Result<(SensorResult, Box<dyn MetricSource>), MetricSourceError>>
            + Send,
    >,
>;

/// Events sent to a metric source worker
#[derive(Debug, Clone, Copy)]
pub enum SourceEvent {
    /// Trigger measurement of metrics
    Measure,

    /// Start a new measurement phase
    NewPhase,

    /// Start a new iteration
    NewIteration,

    /// Enable polling for the source
    StartScheduler,

    /// Disable polling for the source
    StopScheduler,

    /// Signal the worker to finish and join
    JoinWorker,
}

/// Raw phase containing metrics from a metric reader
#[derive(Debug, Default, Clone)]
pub struct RawPhase<V> {
    /// Metrics collected in this raw phase
    pub metrics: V,
}

impl<V> RawPhase<V> {
    /// Create a new RawPhase with the given metrics
    pub fn new(metrics: V) -> Self {
        Self { metrics }
    }
}

/// Represents a single iteration from a metrics source
#[derive(Debug, Default, Clone)]
pub struct RawIteration<V> {
    /// Raw phases collected during the iteration
    pub phases: Vec<RawPhase<V>>,

    /// Total elapsed duration for the iteration
    pub total_elapsed: Duration,

    /// Number of measurements performed
    pub measure_count: u64,
}

impl<V: Into<Metrics>> From<RawIteration<V>> for SensorIteration {
    fn from(iteration: RawIteration<V>) -> Self {
        let phases = iteration
            .phases
            .into_iter()
            .map(|phase| phase.into())
            .collect();

        let measure_delta = if iteration.measure_count > 1 {
            (iteration.total_elapsed.as_micros() / (iteration.measure_count - 1) as u128) as u64
        } else {
            0
        };

        SensorIteration::new(phases, measure_delta, iteration.measure_count)
    }
}
