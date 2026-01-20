use std::fs::File;
use std::io::Write;

use serde_json::json;

use crate::core::displayer::{Displayer, Result, default_iterations_filename};
use crate::core::profiler::types::Iteration;
use crate::core::sensor::Sensor;
use crate::util::file::{create_file_with_user_permissions, get_absolute_path};

/// JSON output writer to file.
pub struct JsonOutput {
    writer: File,
    filename: String,
}

impl Displayer for JsonOutput {
    fn simple_single(&mut self, cmd: &[String], result: &Iteration) -> Result<()> {
        let phase = &result.phases[0];
        let obj = json!({
            "command": cmd.join(" "),
            "mode": "simple",
            "metrics": phase.metrics,
            "duration_ms": phase.duration_ms,
            "exit_code": result.exit_code,
            "measure_count": result.measure_count,
            "measure_delta": result.measure_delta,
        });
        self.write_json(&obj)
    }

    fn simple_iterations(&mut self, cmd: &[String], iterations: &[Iteration]) -> Result<()> {
        let obj = json!({
            "command": cmd.join(" "),
            "mode": "simple-iterations",
            "nb_iterations": iterations.len(),
            "iterations": iterations
        });

        self.write_json(&obj)
    }

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
            "phases": result.phases
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
        self.write_json(&serde_json::to_value(sensors)?)
    }
}

impl JsonOutput {
    /// Creates a JSON output writer to a file.
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

    fn write_json(&mut self, value: &serde_json::Value) -> Result<()> {
        let json_str = serde_json::to_string_pretty(value)?;
        writeln!(self.writer, "{}", json_str)?;
        println!("✔ JSON written to: {}", self.filename);
        Ok(())
    }
}
