use crate::aggregate::Metrics;
use crate::aggregate::iteration::SensorIteration;
use crate::aggregate::sensor_result::SensorResult;
use crate::source::{MetricSource, MetricSourceError};
use std::{fmt::Debug, pin::Pin, time::Duration};

/// Trait for types returned by a [`crate::reader::MetricReader`].
///
/// Any type implementing this trait represents the result of a metric measurement.
/// It must implement `Debug` for logging and debugging, `Send` to be safely
/// transferred across threads, `Default` for easy initialization, and `Into<Metrics>`
/// to allow conversion into the unified [`Metrics`] type used by the profiler.
pub trait MetricReaderTypeBound: Debug + Send + Default + Into<Metrics> {}

impl<T> MetricReaderTypeBound for T where T: Debug + Default + Send + Into<Metrics> {}

/// Trait for errors produced by a [`crate::reader::MetricReader`].
///
/// This trait marks the types of errors that can occur during metric collection.
/// Errors must implement `std::error::Error` for standard Rust error handling
/// and be `Send + Sync` so they can be safely propagated across threads.
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

/// Represents a single iteration from a metrics source
#[derive(Debug, Default, Clone)]
pub struct RawIteration<V> {
    /// Raw phases collected during the iteration
    pub phases: Vec<RawPhase<V>>,

    /// Total elapsed duration for the iteration
    pub total_elapsed: Duration,
}

impl<V: Into<Metrics>> From<RawIteration<V>> for SensorIteration {
    fn from(iteration: RawIteration<V>) -> Self {
        SensorIteration {
            phases: iteration
                .phases
                .into_iter()
                .map(|phase| phase.into())
                .collect(),
        }
    }
}
