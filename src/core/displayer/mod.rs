use crate::{
    config::Config,
    core::{displayer::error::DisplayerError, profiler::types::Iteration, sensor::Sensor},
    output::{OutputFormat, csv::CsvOutput, json::JsonOutput, terminal::TerminalOutput},
    util::time::get_timestamp_micros,
};

pub mod error;

/// Result type for displayer operations
pub type Result<T> = std::result::Result<T, DisplayerError>;

/// Trait for displaying profiler results
pub trait Displayer {
    /// Display a single iteration in simple format
    fn simple_single(&mut self, cmd: &[String], _result: &Iteration) -> Result<()>;

    /// Display multiple iterations in simple format
    fn simple_iterations(&mut self, _cmd: &[String], _results: &[Iteration]) -> Result<()> {
        Err(DisplayerError::NotImplementedForFormat)
    }

    /// Display phases for a single iteration
    fn phases_single(
        &mut self,
        _cmd: &[String],
        _token_pattern: &str,
        _result: &Iteration,
    ) -> Result<()> {
        Err(DisplayerError::NotImplementedForFormat)
    }

    /// Display phases for multiple iterations
    fn phases_iterations(
        &mut self,
        _cmd: &[String],
        _token_pattern: &str,
        _results: &[Iteration],
    ) -> Result<()> {
        Err(DisplayerError::NotImplementedForFormat)
    }

    /// List available sensors
    fn list_sensors(&mut self, _sensors: &[Sensor]) -> Result<()> {
        Err(DisplayerError::NotImplementedForFormat)
    }
}

impl TryFrom<&Config> for Box<dyn Displayer> {
    type Error = DisplayerError;

    /// Creates a displayer from a configuration
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

/// Generates a default filename for iteration data
pub fn default_iterations_filename(ext: &str) -> String {
    format!("data{}.{}", get_timestamp_micros(), ext)
}