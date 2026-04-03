use std::fs::File;
use std::io::Write;

use crate::output::displayer::error::IntoDisplayerError;
use crate::output::displayer::{Displayer, DisplayerError};
use joule_profiler_core::fs::{
    create_file_with_user_permissions, default_iterations_filename, get_absolute_path,
};
use joule_profiler_core::sensor::Sensor;
use joule_profiler_core::types::ProfilerResults;
use serde_json::json;

type Result<T> = std::result::Result<T, DisplayerError>;

/// JSON output writer to a file.
pub struct JsonOutput {
    /// File writer.
    writer: File,

    /// Output filename.
    filename: String,
}

impl JsonOutput {
    /// Create a JSON output writer, optionally with a specific file.
    pub fn new(output_file: Option<String>) -> Result<Self> {
        let filename = output_file.unwrap_or(default_iterations_filename("json"));

        let absolute_path = get_absolute_path(&filename)?;
        let file = create_file_with_user_permissions(&absolute_path)?;

        Ok(Self {
            writer: file,
            filename: absolute_path,
        })
    }

    /// Write a JSON value to the output file.
    fn write_json(&mut self, value: &serde_json::Value) -> Result<()> {
        let json_str = serde_json::to_string_pretty(value)
            .map_err(IntoDisplayerError::into_displayer_error)?;
        writeln!(self.writer, "{json_str}")?;
        println!("JSON written to: {}", self.filename);
        Ok(())
    }
}

impl Displayer for JsonOutput {
    fn display_results(
        &mut self,
        cmd: &[String],
        token_pattern: &str,
        results: &ProfilerResults,
    ) -> Result<()> {
        self.write_json(&json!({
            "command": cmd.join(" "),
            "token_pattern": token_pattern,
            "exit_code": results.exit_code,
            "phases": results.phases,
        }))
    }

    fn list_sensors(&mut self, sensors: &[Sensor]) -> Result<()> {
        self.write_json(
            &serde_json::to_value(sensors).map_err(IntoDisplayerError::into_displayer_error)?,
        )
    }
}
