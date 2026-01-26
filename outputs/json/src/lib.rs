use std::fs::File;
use std::io::Write;

use joule_profiler_core::displayer::error::IntoDisplayerError;
use joule_profiler_core::displayer::{Displayer, DisplayerError};
use joule_profiler_core::profiler::types::Iteration;
use joule_profiler_core::sensor::Sensor;
use joule_profiler_core::util::file::{
    create_file_with_user_permissions, default_iterations_filename, get_absolute_path,
};
use serde_json::json;

type Result<T> = std::result::Result<T, DisplayerError>;

/// JSON output writer to a file
pub struct JsonOutput {
    /// File writer
    writer: File,

    /// Output filename
    filename: String,
}

impl Displayer for JsonOutput {
    fn phases_single(
        &mut self,
        cmd: &[String],
        token_pattern: &str,
        result: &Iteration,
    ) -> Result<()> {
        let obj = json!({
            "command": cmd.join(" "),
            "mode": "phases",
            "token_pattern": token_pattern,
            "exit_code": result.exit_code,
            "phases": result.phases,
        });
        self.write_json(&obj)
    }

    fn phases_iterations(
        &mut self,
        cmd: &[String],
        token_pattern: &str,
        iterations: &[Iteration],
    ) -> Result<()> {
        let root = json!({
            "command": cmd.join(" "),
            "mode": "phases-iterations",
            "token_pattern": token_pattern,
            "nb_iterations": iterations.len(),
            "iterations": iterations
        });
        self.write_json(&root)
    }

    fn list_sensors(&mut self, sensors: &[Sensor]) -> Result<()> {
        self.write_json(
            &serde_json::to_value(sensors).map_err(IntoDisplayerError::into_displayer_error)?,
        )
    }
}

impl JsonOutput {
    /// Create a JSON output writer, optionally with a specific file
    pub fn new(output_file: Option<String>) -> Result<Self> {
        let filename = output_file
            .clone()
            .unwrap_or(default_iterations_filename("json"));

        let absolute_path = get_absolute_path(&filename)?;
        let file = create_file_with_user_permissions(&absolute_path)?;

        Ok(Self {
            writer: file,
            filename: absolute_path,
        })
    }

    /// Write a JSON value to the output file
    fn write_json(&mut self, value: &serde_json::Value) -> Result<()> {
        let json_str = serde_json::to_string_pretty(value)
            .map_err(IntoDisplayerError::into_displayer_error)?;
        writeln!(self.writer, "{}", json_str)?;
        println!("✔ JSON written to: {}", self.filename);
        Ok(())
    }
}
