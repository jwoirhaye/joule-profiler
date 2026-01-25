//! Display profiler results.
//!
//! This module provides abstractions and implementations for displaying
//! JouleProfiler results in various formats, such as terminal output, JSON,
//! or CSV. It also defines the trait [`Displayer`] for custom display implementations.
//!
//! # Overview
//!
//! - [`Displayer`] — Trait defining methods for displaying iterations, phases, and sensors.
//! - [`TerminalOutput`], [`JsonOutput`], [`CsvOutput`] — Standard output formats.
//!
//! # Example
//!
//! ```ignore
//! use joule_profiler::{displayer::Displayer, config::Config};
//!
//! // We assume you have a Config variable
//! let mut displayer: Box<dyn Displayer> = Box::try_from(&config).unwrap();
//!
//! // displayer.simple_single(&cmd, &iteration)?;
//! ```

use crate::{
    config::Config,
    output::{CsvOutput, JsonOutput, OutputFormat, TerminalOutput},
    util::time::get_timestamp_millis,
};

mod error;
use crate::profiler::types::Iteration;
use crate::sensor::Sensor;
pub use error::DisplayerError;

/// Result type for displayer operations.
pub(crate) type Result<T> = std::result::Result<T, DisplayerError>;

/// Trait for displaying profiler results.
///
/// This trait abstracts over different output formats (terminal, JSON, CSV, etc.).
/// Implementors provide methods to display single or multiple iterations, phases,
/// and the list of sensors. Default implementations return
/// [`DisplayerError::NotImplementedForFormat`] if the method is not supported
/// for a given format.
pub trait Displayer {
    /// Display phases for a single iteration.
    ///
    /// Default implementation returns [`DisplayerError::NotImplementedForFormat`].
    ///
    /// # Parameters
    ///
    /// - `_cmd` — Command and arguments that were profiled.
    /// - `_token_pattern` — Regex used to detect phases in output.
    /// - `_result` — Metrics of the iteration to display.
    fn phases_single(
        &mut self,
        _cmd: &[String],
        _token_pattern: &str,
        _result: &Iteration,
    ) -> Result<()> {
        Err(DisplayerError::NotImplementedForFormat)
    }

    /// Display phases for multiple iterations.
    ///
    /// Default implementation returns [`DisplayerError::NotImplementedForFormat`].
    ///
    /// # Parameters
    ///
    /// - `_cmd` — Command and arguments that were profiled.
    /// - `_token_pattern` — Regex used to detect phases in output.
    /// - `_results` — Metrics of the iterations to display.
    fn phases_iterations(
        &mut self,
        _cmd: &[String],
        _token_pattern: &str,
        _results: &[Iteration],
    ) -> Result<()> {
        Err(DisplayerError::NotImplementedForFormat)
    }

    /// List available sensors.
    ///
    /// Default implementation returns [`DisplayerError::NotImplementedForFormat`].
    ///
    /// # Parameters
    ///
    /// - `_sensors` — Slice of sensors to list.
    fn list_sensors(&mut self, _sensors: &[Sensor]) -> Result<()> {
        Err(DisplayerError::NotImplementedForFormat)
    }
}

impl TryFrom<&Config> for Box<dyn Displayer> {
    type Error = DisplayerError;

    /// Creates a boxed [`Displayer`] from a [`Config`] object.
    ///
    /// Chooses the appropriate output format based on `Config::output_format`
    /// and returns an error if initialization fails (e.g., file creation for JSON/CSV).
    fn try_from(config: &Config) -> Result<Self> {
        let output_file = config.output_file.clone();
        let displayer = match config.output_format {
            OutputFormat::Terminal => TerminalOutput.into(),
            OutputFormat::Json => JsonOutput::new(output_file)?.into(),
            OutputFormat::Csv => CsvOutput::try_new(output_file)?.into(),
        };
        Ok(displayer)
    }
}

impl<T: Displayer + 'static> From<T> for Box<dyn Displayer> {
    /// Boxes a displayer for dynamic dispatch
    fn from(displayer: T) -> Self {
        Box::new(displayer)
    }
}

impl Default for Box<dyn Displayer> {
    /// Returns a boxed [`TerminalOutput`] as the default displayer.
    fn default() -> Self {
        TerminalOutput.into()
    }
}

/// Generates a default filename for iteration data
pub(crate) fn default_iterations_filename(ext: &str) -> String {
    format!("data{}.{}", get_timestamp_millis(), ext)
}
