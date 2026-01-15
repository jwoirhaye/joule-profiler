use anyhow::Result;

use crate::{
    config::{ListSensorsConfig, ProfileConfig},
    measurement::{MeasurementResult, PhaseMeasurementResult},
    output::OutputFormatTrait,
    source::{Metric, Sensor},
};

/// Constants for formatting
const BORDER_DOUBLE: &str = "═";
const BORDER_SINGLE: &str = "─";
const BOX_WIDTH: usize = 50;

#[derive(Debug, Clone, Default)]
pub struct TerminalOutput;

impl OutputFormatTrait for TerminalOutput {
    fn simple_single(&mut self, config: &ProfileConfig, result: &MeasurementResult) -> Result<()> {
        self.display_command(&config.cmd);
        self.display_result(&result.metrics, "")
    }

    fn simple_iterations(
        &mut self,
        config: &ProfileConfig,
        results: &[MeasurementResult],
    ) -> Result<()> {
        self.display_command(&config.cmd);

        for (idx, result) in results.iter().enumerate() {
            self.display_iteration_header(idx, results.len());
            self.display_result(&result.metrics, "")?;
        }

        Ok(())
    }

    fn phases_single(
        &mut self,
        config: &ProfileConfig,
        result: &PhaseMeasurementResult,
    ) -> Result<()> {
        self.display_command(&config.cmd);

        for phase in result.phases.iter() {
            self.display_phase_header(
                &phase.name,
                phase.start_token.as_deref(),
                phase.end_token.as_deref(),
                phase.start_line,
                phase.end_line,
                "",
            );
            self.display_result(&phase.metrics, "")?;
        }

        Ok(())
    }

    fn phases_iterations(
        &mut self,
        config: &ProfileConfig,
        results: &[PhaseMeasurementResult],
    ) -> Result<()> {
        if results.is_empty() {
            return Ok(());
        }

        self.display_command(&config.cmd);

        for (idx, iteration_results) in results.iter().enumerate() {
            self.display_iteration_header(idx, results.len());

            for phase in &iteration_results.phases {
                self.display_phase_header(
                    &phase.name,
                    phase.start_token.as_deref(),
                    phase.end_token.as_deref(),
                    phase.start_line,
                    phase.end_line,
                    "  ",
                );
                self.display_result(&phase.metrics, "  ")?;
            }
        }

        Ok(())
    }

    fn list_sensors(&mut self, _config: &ListSensorsConfig, sensors: &[Sensor]) -> Result<()> {
        if sensors.is_empty() {
            println!("No sensors available.");
            return Ok(());
        }

        self.print_header("Available Sensors");

        println!("  {:<20} | {:<10} | {:<15}", "Name", "Unit", "Source");
        println!("  {}", BORDER_SINGLE.repeat(45));

        for sensor in sensors {
            println!(
                "  {:<20} | {:<10} | {:<15}",
                sensor.name, sensor.unit, sensor.source
            );
        }

        println!("{}", BORDER_DOUBLE.repeat(BOX_WIDTH));

        Ok(())
    }
}

impl TerminalOutput {
    /// Display command header
    fn display_command(&self, command: &[String]) {
        if !command.is_empty() {
            println!();
            self.print_header("Command");
            println!("  {}", command.join(" "));
        }
    }

    /// Print a formatted header
    fn print_header(&self, title: &str) {
        println!("╔{}╗", BORDER_DOUBLE.repeat(BOX_WIDTH));
        println!("║  {:<width$} ║", title, width = BOX_WIDTH - 3);
        println!("╚{}╝", BORDER_DOUBLE.repeat(BOX_WIDTH));
    }

    /// Print a formatted sub-header
    fn print_subheader(&self, title: &str, prefix: &str) {
        println!(
            "{}┌{}┐",
            prefix,
            BORDER_SINGLE.repeat(BOX_WIDTH - prefix.len())
        );
        println!(
            "{}│ {:<width$}│",
            prefix,
            title,
            width = BOX_WIDTH - prefix.len() - 3
        );
        println!(
            "{}└{}┘",
            prefix,
            BORDER_SINGLE.repeat(BOX_WIDTH - prefix.len())
        );
    }

    /// Display a single measurement result
    fn display_result(&self, metrics: &[Metric], prefix: &str) -> Result<()> {
        println!();
        println!("{}{}", prefix, BORDER_DOUBLE.repeat(BOX_WIDTH));

        let mut keys: Vec<_> = metrics.iter().map(|metric| &metric.name).cloned().collect();
        keys.sort_unstable();

        for metric in metrics {
            println!(
                "{}  {:<20}: {:10.6} {}",
                prefix, metric.name, metric.value, metric.unit
            );
        }

        // println!("{}  {:<20}: {:>10.6} s", prefix, "Duration", duration_s);
        // println!("{}  {:<20}: {:>10}", prefix, "Exit code", res.exit_code);
        println!("{}{}", prefix, BORDER_DOUBLE.repeat(BOX_WIDTH));

        Ok(())
    }

    /// Display iteration header
    fn display_iteration_header(&self, idx: usize, total: usize) {
        println!("\n╔{}╗", BORDER_DOUBLE.repeat(BOX_WIDTH));
        println!(
            "║  Iteration {} / {:<width$} ║",
            idx + 1,
            total,
            width = BOX_WIDTH - 17
        );
        println!("╚{}╝", BORDER_DOUBLE.repeat(BOX_WIDTH));
    }

    /// Display phase header with token information
    fn display_phase_header(
        &self,
        phase_name: &str,
        start_token: Option<&str>,
        end_token: Option<&str>,
        start_line: Option<usize>,
        end_line: Option<usize>,
        prefix: &str,
    ) {
        println!();
        if prefix.is_empty() {
            println!("╔{}╗", BORDER_DOUBLE.repeat(BOX_WIDTH));
            println!("║  Phase: {:<width$} ║", phase_name, width = BOX_WIDTH - 10);
            println!("╚{}╝", BORDER_DOUBLE.repeat(BOX_WIDTH));
        } else {
            self.print_subheader(&format!("Phase: {}", phase_name), prefix);
        }

        // Display token information
        if let Some(start) = start_token {
            let start_info = if let Some(line) = start_line {
                format!("{} (line {})", start, line)
            } else {
                start.to_string()
            };
            println!("{}  Start token: {}", prefix, start_info);
        }

        if let Some(end) = end_token {
            let end_info = if let Some(line) = end_line {
                format!("{} (line {})", end, line)
            } else {
                end.to_string()
            };
            println!("{}  End token  : {}", prefix, end_info);
        }
    }
}
