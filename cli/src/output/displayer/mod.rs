//! Display profiler results.
//!
//! This module provides abstractions and implementations for displaying
//! JouleProfiler results in various formats, such as terminal output, JSON,
//! or CSV. It also defines the trait [`Displayer`] for custom display implementations.
//!
//! # Overview
//!
//! - [`Displayer`] — Trait defining methods for displaying iterations, phases, and sensors.
//! - [`Terminal`][`super::formats::terminal::TerminalOutput`], [`JsonOutput`][`super::formats::json::JsonOutput`], [`CsvOutput`][`super::formats::csv::CsvOutput`] — Standard output formats.

pub mod error;
pub use error::DisplayerError;
use joule_profiler_core::{sensor::Sensor, types::Iteration};

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
    /// Display iteration(s) results
    ///
    /// # Parameters
    ///
    /// - `cmd` — Command and arguments that were profiled.
    /// - `token_pattern` — Regex used to detect phases in output.
    /// - `results` — Metrics of the iterations to display.
    fn profile(
        &mut self,
        cmd: &[String],
        token_pattern: &str,
        results: &[Iteration],
    ) -> Result<()> {
        if results.len() > 1 {
            self.phases_iterations(cmd, token_pattern, results)
        } else if let Some(result) = results.first() {
            self.phases_single(cmd, token_pattern, result)
        } else {
            Err(DisplayerError::NoIterationFound)
        }
    }

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

impl<T: Displayer + 'static> From<T> for Box<dyn Displayer> {
    /// Boxes a displayer for dynamic dispatch
    fn from(displayer: T) -> Self {
        Box::new(displayer)
    }
}
