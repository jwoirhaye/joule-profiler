use std::fs::File;
use std::io::Write;

use anyhow::{Result, bail};
use log::{info, trace};
use serde_json::json;

use crate::config::{ListSensorsConfig, Mode, ProfileConfig};
use crate::measurement::{MeasurementResult, PhaseMeasurementResult};
use crate::output::{OutputFormatTrait, default_iterations_filename};
use crate::source::Sensor;
use crate::util::file::{create_file_with_user_permissions, get_absolute_path};

/// JSON output writer to file.
pub struct JsonOutput {
    writer: File,
    filename: String,
}

impl OutputFormatTrait for JsonOutput {
    fn simple_single(&mut self, config: &ProfileConfig, result: &MeasurementResult) -> Result<()> {
        let obj = json!({
            "command": config.cmd.join(" "),
            "mode": "simple",
            "metrics": result.metrics,
            "duration_ms": result.duration_ms,
            "exit_code": result.exit_code,
            "measure_count": result.measure_count,
            "measure_delta": result.measure_delta,
        });

        self.write_json(&obj)
    }

    fn simple_iterations(
        &mut self,
        config: &ProfileConfig,
        results: &[MeasurementResult],
    ) -> Result<()> {
        info!("Formatting {} simple iterations", results.len());

        let iters: Vec<_> = results
            .iter()
            .enumerate()
            .map(|(idx, result)| {
                trace!("Formatting iteration {}", idx + 1);
                json!({
                    "iteration": idx + 1,
                    "metrics": result.metrics,
                    "duration_ms": result.duration_ms,
                    "exit_code": result.exit_code,
                    "measure_count": result.measure_count,
                    "measure_delta": result.measure_delta,
                })
            })
            .collect();

        let root = json!({
            "command": config.cmd.join(" "),
            "mode": "simple-iterations",
            "iterations": iters
        });

        self.write_json(&root)
    }

    fn phases_single(
        &mut self,
        config: &ProfileConfig,
        result: &PhaseMeasurementResult,
    ) -> Result<()> {
        let phases_value = serde_json::to_value(result.phases.clone())?;
        let phases_config = match &config.mode {
            Mode::SimpleMode => bail!("Invalid configuration mode."),
            Mode::PhaseMode(phases_config) => phases_config,
        };

        let obj = json!({
            "command": config.cmd.join(" "),
            "mode": "phases",
            "token_pattern": phases_config.token_pattern,
            "exit_code": result.exit_code,
            "phases": phases_value
        });

        self.write_json(&obj)
    }

    fn phases_iterations(
        &mut self,
        config: &ProfileConfig,
        results: &[PhaseMeasurementResult],
    ) -> Result<()> {
        info!("Formatting {} phase iterations", results.len());
        let phases_config = match &config.mode {
            Mode::SimpleMode => bail!("Invalid configuration mode."),
            Mode::PhaseMode(phases_config) => phases_config,
        };

        let iters: Vec<_> = results
            .iter()
            .enumerate()
            .map(|(idx, result)| {
                json!({
                    "iteration": idx + 1,
                    "exit_code": result.exit_code,
                    "duration": result.duration_ms,
                    "phases": result.phases,
                })
            })
            .collect();

        let root = json!({
            "command": config.cmd.join(" "),
            "mode": "phases-iterations",
            "token_pattern": phases_config.token_pattern,
            "iterations": iters
        });

        self.write_json(&root)
    }

    fn list_sensors(&mut self, _config: &ListSensorsConfig, sensors: &[Sensor]) -> Result<()> {
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
        info!("Creating JSON output file: {}", absolute_path);

        let file = create_file_with_user_permissions(&absolute_path)?;

        Ok(Self {
            writer: file,
            filename: absolute_path,
        })
    }

    fn write_json(&mut self, value: &serde_json::Value) -> Result<()> {
        let json_str = serde_json::to_string_pretty(value)?;
        trace!("Writing JSON output ({} bytes)", json_str.len());
        writeln!(self.writer, "{}", json_str)?;

        println!("✔ JSON written to: {}", self.filename);
        info!("JSON output saved to: {}", self.filename);

        Ok(())
    }
}
