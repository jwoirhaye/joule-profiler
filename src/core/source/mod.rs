use std::{fmt::Debug, time::Duration};

use tokio::sync::mpsc::Receiver;

use crate::core::{
    metric::Metrics,
    phase::SourcePhase,
    sensor::Sensors,
    source::{
        accumulator::MetricAccumulator,
        error::MetricSourceError,
        reader::MetricReader,
        types::{MetricSourceWorkerFuture, SensorIteration, SourceEvent},
    },
};

pub mod accumulator;
pub mod error;
pub mod reader;
pub mod result;
pub mod types;

#[derive(Debug, Default, Clone)]
struct SourceIteration<V> {
    pub phases: Vec<SourcePhase<V>>,
    pub total_elapsed: Duration,
    pub measure_count: u64,
}

impl<V: Into<Metrics>> From<SourceIteration<V>> for SensorIteration {
    fn from(iteration: SourceIteration<V>) -> Self {
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

pub trait MetricSource: Send {
    /// Runs the worker and returns the result along with the source itself.
    fn run(self: Box<Self>, rx: Receiver<SourceEvent>) -> MetricSourceWorkerFuture;

    fn list_sensors(&self) -> Result<Sensors, MetricSourceError>;

    fn into_box(self) -> Box<dyn MetricSource>
    where
        Self: Sized + 'static,
    {
        Box::new(self)
    }
}

impl<T> MetricSource for MetricAccumulator<T>
where
    T: MetricReader,
{
    fn run(self: Box<Self>, rx: Receiver<SourceEvent>) -> MetricSourceWorkerFuture {
        Box::pin(async move { self.run_worker(rx).await })
    }

    fn list_sensors(&self) -> Result<Sensors, MetricSourceError> {
        self.get_sensors()
    }
}

impl<T> From<T> for Box<dyn MetricSource>
where
    T: MetricReader,
{
    fn from(reader: T) -> Self {
        let source = MetricAccumulator::new(reader);
        Box::new(source)
    }
}
