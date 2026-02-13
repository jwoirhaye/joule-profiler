//! Internal abstractions for metric sources.
//!
//! This module defines the private traits and runtime glue used by the
//! profiler to manage metric readers. It is **not** part of the public API.
//! Implementations are boxed for flexibility, while internally retaining
//! concrete types for zero-cost metric operations.

use std::sync::Arc;
use std::sync::atomic::AtomicI32;

use tokio::sync::mpsc::{Sender, channel};

pub(crate) mod accumulator;
pub mod error;
pub mod reader;
pub(crate) mod runtime;
pub(crate) mod types;

use crate::sensor::Sensors;
use crate::source::runtime::MetricSourceRuntime;
use crate::source::types::{SourceEvent, SourceWorkerHandle};
pub use error::MetricSourceError;
pub use reader::MetricReader;
pub use types::{MetricReaderErrorBound, MetricReaderTypeBound};

/// Internal trait representing a runnable metric source.
///
/// Implemented by the runtime wrapper around a [`MetricReader`].
pub(crate) trait MetricSource: Send {
    /// Spawn the source worker and return its handle and control channel.
    ///
    /// The pid argument refers to the profiled program pid, default is zero but it is updated on each iteration
    fn run(self: Box<Self>, pid: Arc<AtomicI32>) -> (SourceWorkerHandle, Sender<SourceEvent>);

    /// List sensors exposed by this source.
    fn list_sensors(&self) -> Result<Sensors, MetricSourceError>;
}

impl<R> MetricSource for MetricSourceRuntime<R>
where
    R: MetricReader,
{
    fn run(self: Box<Self>, pid: Arc<AtomicI32>) -> (SourceWorkerHandle, Sender<SourceEvent>) {
        let (tx, rx) = channel(4);
        let handle = tokio::spawn(async move { self.run_worker(rx, pid).await });
        (handle, tx)
    }

    fn list_sensors(&self) -> Result<Sensors, MetricSourceError> {
        self.get_source_sensors()
    }
}

impl<R> From<R> for Box<dyn MetricSource>
where
    R: MetricReader,
{
    fn from(reader: R) -> Self {
        let source = MetricSourceRuntime::new(reader);
        Box::new(source)
    }
}
