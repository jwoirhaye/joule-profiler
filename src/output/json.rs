use std::fs::File;
use std::io::Write;

use anyhow::Result;
use log::{info, trace};
use serde_json::json;

use crate::config::Config;
use crate::measure::{MeasurementResult, PhasesResult};

use super::{OutputFormat, default_iterations_filename, get_absolute_path};

/// JSON output writer to file.
pub struct JsonOutput {
    writer: File,
    filename: String,
}

impl JsonOutput {
    /// Creates a JSON output writer to a file.
    pub fn new(config: &Config) -> Result<Self> {
        let filename = config
            .jouleit_file
            .clone()
            .unwrap_or_else(|| default_iterations_filename("json"));

        let absolute_path = get_absolute_path(&filename)?;
        info!("Creating JSON output file: {}", absolute_path);

        let file = File::create(&filename)?;

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

impl OutputFormat for JsonOutput {
    fn simple_single(&mut self, _config: &Config, res: &MeasurementResult) -> Result<()> {
        let obj = json!({
            "command": _config.cmd.join(" "),
            "mode": "simple",
            "energy_uj": res.energy_uj,
            "duration_ms": res.duration_ms,
            "exit_code": res.exit_code
        });

        self.write_json(&obj)
    }

    fn simple_iterations(
        &mut self,
        _config: &Config,
        results: &[(usize, MeasurementResult)],
    ) -> Result<()> {
        info!("Formatting {} simple iterations", results.len());

        let iters: Vec<_> = results
            .iter()
            .map(|(idx, res)| {
                trace!("Formatting iteration {}", idx);
                json!({
                    "iteration": idx,
                    "energy_uj": res.energy_uj,
                    "duration_ms": res.duration_ms,
                    "exit_code": res.exit_code
                })
            })
            .collect();

        let root = json!({
            "command": _config.cmd.join(" "),
            "mode": "simple-iterations",
            "iterations": iters
        });

        self.write_json(&root)
    }

    fn phases_single(&mut self, config: &Config, phases: &PhasesResult) -> Result<()> {
        let phases_value = serde_json::to_value(&phases.phases)?;

        let obj = json!({
            "command": config.cmd.join(" "),
            "mode": "phases",
            "token_start": config.token_start,
            "token_end": config.token_end,
            "phases": phases_value
        });

        self.write_json(&obj)
    }

    fn phases_iterations(
        &mut self,
        config: &Config,
        results: &[(usize, PhasesResult)],
    ) -> Result<()> {
        info!("Formatting {} phase iterations", results.len());

        let iters: Vec<_> = results
            .iter()
            .map(|(idx, phases)| {
                trace!(
                    "Formatting iteration {} ({} phases)",
                    idx,
                    phases.phases.len()
                );
                json!({
                    "iteration": idx,
                    "phases": phases.phases,
                })
            })
            .collect();

        let root = json!({
            "command": config.cmd.join(" "),
            "mode": "phases-iterations",
            "token_start": config.token_start,
            "token_end": config.token_end,
            "iterations": iters
        });

        self.write_json(&root)
    }
}
