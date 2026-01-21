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
//! into [`metrics::Metric`] and [`metrics::Metrics`] objects **after the measurement phase is finished**.
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
//!     JouleProfiler::new().run().await
//! }
//! ```
//!
//! # Extending
//!
//! To add a new metric source, you must meet a few requirements:
//! - The source structure must implement the [`reader::MetricReader`] trait.
//! - The associated type MetricReader::Type must implement the [`reader::MetricReaderTypeBound`] traits.
//! - The associated type MetricReader::Error must implement the [`reader::MetricReaderErrorBound`] traits.
//!
//! # Error Handling
//!
//! Errors are considered fatal and stop the profiler by default.

use anyhow::Result;
use log::info;

use crate::{cli::Cli, config::Config, util::logging::init_logging};

pub mod cli;
pub mod config;
mod core;
pub mod output;
pub mod sources;
mod util;

pub use core::aggregate as metrics;
pub use core::displayer;
pub use core::sensor;
pub use core::{JouleProfiler, JouleProfilerError, source as reader};

/// Initialize and run the Joule Profiler.
///
/// This function performs all necessary setup to start the profiler:
/// 1. Parses the command-line arguments using [`Cli::from_args`].
/// 2. Initializes logging according to the verbosity level specified by the user.
/// 3. Builds the profiler configuration from the CLI arguments.
/// 4. Instantiates the [`JouleProfiler`] and starts its measurements.
///
/// # Errors
///
/// Returns an [`anyhow::Result`] error if:
/// - CLI parsing fails
/// - Logging initialization fails
/// - Profiler construction fails
/// - An error is encountered during the profiler execution
///
/// All errors are propagated upwards, and the profiler will stop if any
/// step fails.
///
/// # Examples
///
/// ```no_run
/// use joule_profiler::run;
/// use anyhow::Result;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///    joule_profiler::run().await
/// }
/// ```
pub async fn run() -> Result<()> {
    let cli = Cli::from_args()?;
    init_logging(cli.verbose);

    let config = Config::from(cli);

    info!("Joule Profiler starting");
    let mut profiler = JouleProfiler::try_from(config)?;
    profiler.run().await?;

    Ok(())
}
