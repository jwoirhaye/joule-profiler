//! Core module for metric sources in JouleProfiler.
//!
//! This module provides the abstractions and utilities for defining,
//! running, and managing metric sources. Metric sources represent the
//! origin of measurements (e.g., RAPL, CPU counters) that the profiler
//! collects, aggregates, and exports.
//!
//! # Public API
//!
//! The following items are publicly accessible:
//! - [`MetricSource`] — Trait representing a metric source to be used with the profiler.
//! - [`MetricReader`] — Trait implemented by types that can read raw metrics from a source.
//! - [`MetricSourceError`] — Error type used for all metric source operations.
//! - [`MetricReaderTypeBound`] — Bound for metric types returned by readers.
//! - [`MetricReaderErrorBound`] — Bound for errors produced by readers.
//!
//! # Key Concepts
//!
//! - **MetricSource**  
//!   Represents a source of metrics. It can be run asynchronously via the
//!   [`MetricSource::run`] method, and its available sensors can be listed with [`MetricSource::list_sensors`].
//!
//! - **MetricAccumulator**  
//!   A generic adapter that wraps a [`MetricReader`] and implements [`MetricSource`].
//!   It handles scheduling, polling, and aggregation of metrics from the reader.
//!
//!   Importantly, it **monomorphizes the MetricSource**: although the profiler
//!   stores sources as `Box<dyn MetricSource>` for flexibility, internally each
//!   source retains its concrete type for zero-cost statically-typed metric
//!   operations. This design allows runtime polymorphism without sacrificing
//!   performance.
//!
//! - **MetricReader**  
//!   Implemented by any type that can produce metrics. Readers define:
//!   - The type of metrics they return (`Type`).
//!   - The error type they may produce (`Error`) which **must implement `Error`**.
//!   - Methods to measure, retrieve, and list available sensors.
//!
//! # Usage
//!
//! A new source can be added by implementing [`MetricReader`] and converting it
//! into a [`MetricSource`] via `Box::from(reader)`. The profiler then runs the
//! source asynchronously and aggregates its metrics.
//!
//! ```no_run
//! use joule_profiler_core::{
//!     sensor::Sensors,
//!     source::{MetricSource, MetricSourceError,MetricReader},
//!     aggregate::{Metric, Metrics},
//! };
//!
//! use std::vec::Vec;
//!
//! struct MyReader;
//!
//! #[derive(Debug, Default)]
//! struct MyReaderType {
//!     value: u64
//! }
//!
//! impl From<MyReaderType> for Metrics {
//!     fn from(reader_type: MyReaderType) -> Self {
//!         let metric = Metric::new("value".into(), reader_type.value, "unit".into(), "MyReader".into());
//!         vec![metric]
//!     }
//! }
//!
//! impl MetricReader for MyReader {
//!     type Type = MyReaderType;
//!     type Error = MetricSourceError; // Or any type that implement std::error::Error
//!
//!     fn measure(&mut self) -> Result<(), Self::Error> { Ok(()) }
//!     fn retrieve(&mut self) -> Result<Self::Type, Self::Error> { Ok(MyReaderType { value: 42 }) }
//!     fn get_sensors(&self) -> Result<Sensors, Self::Error> { Ok(Vec::new()) }
//!     fn get_name() -> &'static str { "MyReader" }
//! }
//!
//! let source: Box<dyn MetricSource> = Box::from(MyReader);
//! ```
//!
//! # Notes
//!
//! - Metric sources are **lazy**: metrics are only aggregated **after the measurement
//!   phase** to avoid runtime overhead.
//! - An hidden `MetricAccumulator` provides a generic implementation for most readers, so
//!   implementing [`MetricSource`] manually is rarely necessary.
//! - Monomorphization ensures that although sources are stored as trait objects,
//!   the internal metric operations remain statically typed and efficient to minimize overhead.

use tokio::sync::mpsc::Receiver;

pub(crate) mod accumulator;
pub mod error;
pub(crate) mod reader;
pub(crate) mod types;

use crate::sensor::Sensors;
use crate::source::accumulator::MetricAccumulator;
use crate::source::types::{MetricSourceFuture, SourceEvent};
pub use error::MetricSourceError;
pub use reader::MetricReader;
pub use types::{MetricReaderErrorBound, MetricReaderTypeBound};

/// Trait representing a metric source and required to be used in profiler
pub trait MetricSource: Send {
    /// Runs the worker and returns a future that resolves with the result and the source itself
    fn run(self: Box<Self>, rx: Receiver<SourceEvent>) -> MetricSourceFuture;

    /// List all sensors available from this source
    fn list_sensors(&self) -> Result<Sensors, MetricSourceError>;
}

impl<R> MetricSource for MetricAccumulator<R>
where
    R: MetricReader,
{
    /// Run the worker for the metric accumulator
    fn run(self: Box<Self>, rx: Receiver<SourceEvent>) -> MetricSourceFuture {
        Box::pin(async move { self.run_worker(rx).await })
    }

    /// List all sensors for this accumulator
    fn list_sensors(&self) -> Result<Sensors, MetricSourceError> {
        self.get_sensors()
    }
}

impl<R> From<R> for Box<dyn MetricSource>
where
    R: MetricReader,
{
    fn from(reader: R) -> Self {
        let source = MetricAccumulator::new(reader);
        Box::new(source)
    }
}
