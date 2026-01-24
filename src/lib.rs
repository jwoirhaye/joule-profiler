//! JouleProfiler — Energy profiling of programs.
//!
//! JouleProfiler provides a modular and extensible framework
//! for collecting, aggregating, and exporting energy metrics
//! from multiple sources. It also provides a clean interface
//! to implement new metric sources easily.
//!
//! # Architecture
//!
//! JouleProfiler is designed for modularity and performance:
//! - It relies on dynamic traits for modularity and extensibility
//! - It resolves metric sources at runtime and transforms them into static types known at compile time
//! - Errors are propagated through the profiler with well-defined boundaries for debugging purposes
//!
//! Metrics are collected from sources during program execution, but **they are only aggregated**
//! into [`Metric`](`metrics::Metric`) and [`Metrics`](`metrics::Metrics`) objects **after the measurement phase is finished**.
//! This ensures that the profiler introduces minimal runtime overhead while collecting data.
//!
//! This design allows you to easily implement and plug in multiple sources,
//! and extend its functionalities. It maintains low overhead, which is crucial
//! when measuring energy consumption and system metrics.
//!
//! # Getting Started
//!
//! ```no_run
//! use joule_profiler::{JouleProfiler, JouleProfilerError};
//!
//! pub async fn run() -> Result<(), JouleProfilerError> {
//!     JouleProfiler::from_cli()?.run().await
//! }
//! ```
//!
//! # Extending
//!
//! To add a new metric source, you must meet a few requirements:
//! - The source structure must implement the [`MetricReader`](`reader::MetricReader`) trait.
//! - The associated type MetricReader::Type must implement the [`MetricReaderTypeBound`](`reader::MetricReaderTypeBound`) traits.
//! - The associated type MetricReader::Error must implement the [`MetricReaderErrorBound`](`reader::MetricReaderErrorBound`) traits.
//!
//! # Error Handling
//!
//! Errors are considered fatal and stop the profiler by default.

pub mod cli;
pub mod config;
mod core;
pub mod output;
pub mod sources;
mod util;

pub use core::aggregate as metrics;
pub use core::displayer;
pub use core::sensor;
pub use core::{source as reader, JouleProfiler, JouleProfilerError};
