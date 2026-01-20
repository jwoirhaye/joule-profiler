use tokio::sync::mpsc::Receiver;

use crate::core::{
    sensor::Sensors,
    source::{
        accumulator::MetricAccumulator,
        error::MetricSourceError,
        reader::MetricReader,
        types::{MetricSourceWorkerFuture, SourceEvent},
    },
};

pub mod accumulator;
pub mod error;
pub mod reader;
pub mod result;
pub mod types;

/// Trait representing a metric source and required to be used in profiler 
pub trait MetricSource: Send {
    /// Runs the worker and returns a future that resolves with the result and the source itself
    fn run(self: Box<Self>, rx: Receiver<SourceEvent>) -> MetricSourceWorkerFuture;

    /// List all sensors available from this source
    fn list_sensors(&self) -> Result<Sensors, MetricSourceError>;

    /// Convert into a boxed trait object
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
    /// Run the worker for the metric accumulator
    fn run(self: Box<Self>, rx: Receiver<SourceEvent>) -> MetricSourceWorkerFuture {
        Box::pin(async move { self.run_worker(rx).await })
    }

    /// List all sensors for this accumulator
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
