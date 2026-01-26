use std::collections::HashMap;

use joule_profiler_core::{
    aggregate::Metric,
    displayer::{Displayer, DisplayerError},
    profiler::types::{Iteration, Phase},
    sensor::Sensor,
};

/// Constants for formatting
const BORDER_DOUBLE: &str = "═";
const BORDER_SINGLE: &str = "─";
const BOX_WIDTH: usize = 50;

type Result<T> = std::result::Result<T, DisplayerError>;

#[derive(Debug, Clone, Default)]
pub struct TerminalOutput;

impl TerminalOutput {
    /// Display command header
    fn display_command(&self, command: &[String]) {
        if !command.is_empty() {
            self.print_header("Command");
            println!("  {}", command.join(" "));
        }
    }

    /// Print a formatted header
    fn print_header(&self, title: &str) {
        println!("╔{}╗", BORDER_DOUBLE.repeat(BOX_WIDTH - 2));
        println!("║  {:<width$} ║", title, width = BOX_WIDTH - 5);
        println!("╚{}╝", BORDER_DOUBLE.repeat(BOX_WIDTH - 2));
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
            width = BOX_WIDTH - prefix.len() - 1
        );
        println!(
            "{}└{}┘",
            prefix,
            BORDER_SINGLE.repeat(BOX_WIDTH - prefix.len())
        );
    }

    /// Display a single measurement result
    fn display_phase(&self, phase: &Phase, prefix: &str) -> Result<()> {
        println!();

        let mut keys: Vec<_> = phase
            .metrics
            .iter()
            .map(|metric| &metric.name)
            .cloned()
            .collect();
        keys.sort_unstable();

        let mut metrics_per_source: HashMap<&String, Vec<&Metric>> = HashMap::new();
        for metric in &phase.metrics {
            metrics_per_source
                .entry(&metric.source)
                .or_default()
                .push(metric)
        }

        for metrics in metrics_per_source.values() {
            println!(" {}", BORDER_SINGLE.repeat(BOX_WIDTH - 2));
            for metric in metrics {
                println!(
                    "{}  {:<20}: {:10.6} {}",
                    prefix, metric.name, metric.value, metric.unit
                );
            }
        }

        Ok(())
    }

    fn display_iteration(&self, iteration: &Iteration, prefix: &str) -> Result<()> {
        println!(
            "{}  {:<20}: {:>10} ms",
            prefix, "Duration", iteration.duration_ms
        );
        println!(
            "{}  {:<20}: {:>10}",
            prefix, "Exit code", iteration.exit_code
        );

        for phase in &iteration.phases {
            self.display_phase_header(phase, prefix);
            self.display_phase(phase, prefix)?;
        }

        Ok(())
    }

    /// Display iteration header
    fn display_iteration_header(&self, idx: usize, total: usize) {
        println!("\n╔{}╗", BORDER_DOUBLE.repeat(BOX_WIDTH - 2));
        println!(
            "║  Iteration {} / {:<width$} ║",
            idx + 1,
            total,
            width = BOX_WIDTH - 19
        );
        println!("╚{}╝", BORDER_DOUBLE.repeat(BOX_WIDTH - 2));
    }

    /// Display phase header with token information
    fn display_phase_header(&self, phase: &Phase, prefix: &str) {
        let phase_name = phase.get_name();
        println!();
        if prefix.is_empty() {
            println!("╔{}╗", BORDER_DOUBLE.repeat(BOX_WIDTH - 2));
            println!("║  Phase: {:<width$} ║", phase_name, width = BOX_WIDTH - 12);
            println!("╚{}╝", BORDER_DOUBLE.repeat(BOX_WIDTH - 2));
        } else {
            self.print_subheader(&format!("Phase: {}", phase_name), prefix);
        }

        // Display token information
        let start_info = if let Some(line) = phase.start_line {
            format!("{} (line {})", phase.start_token, line)
        } else {
            phase.start_token.to_string()
        };

        let end_info = if let Some(line) = phase.end_line {
            format!("{} (line {})", phase.end_token, line)
        } else {
            phase.end_token.to_string()
        };

        println!(
            "{}  {:<20}: {:>10} ms",
            prefix, "Duration", phase.duration_ms
        );

        println!("{}  {:<20}: {:>10}", prefix, "Start token", start_info);

        println!("{}  {:<20}: {:>10}", prefix, "End token", end_info);
    }
}

impl Displayer for TerminalOutput {
    fn phases_single(
        &mut self,
        cmd: &[String],
        _token_pattern: &str,
        iteration: &Iteration,
    ) -> Result<()> {
        self.display_command(cmd);
        println!(" {}", BORDER_SINGLE.repeat(BOX_WIDTH));
        self.display_iteration(iteration, "")?;

        Ok(())
    }

    fn phases_iterations(
        &mut self,
        cmd: &[String],
        _token_pattern: &str,
        iterations: &[Iteration],
    ) -> Result<()> {
        self.display_command(cmd);
        let nb_iterations = iterations.len();

        for (idx, iteration) in iterations.iter().enumerate() {
            self.display_iteration_header(idx, nb_iterations);
            self.display_iteration(iteration, "")?;
            println!();
        }

        Ok(())
    }

    fn list_sensors(&mut self, sensors: &[Sensor]) -> Result<()> {
        if sensors.is_empty() {
            println!("No sensors available.");
            return Ok(());
        }
        self.print_header("Available Sensors");

        println!("  {:<20} | {:<5} | {:<15}", "Name", "Unit", "Source");

        let mut sensors_by_source: HashMap<&String, Vec<&Sensor>> = HashMap::new();
        for sensor in sensors {
            sensors_by_source
                .entry(&sensor.source)
                .or_default()
                .push(sensor)
        }

        for source_sensors in sensors_by_source.values() {
            println!(" {}", BORDER_SINGLE.repeat(BOX_WIDTH - 2));
            for sensor in source_sensors {
                println!(
                    "  {:<20} | {:<5} | {:<15}",
                    sensor.name, sensor.unit, sensor.source
                );
            }
        }

        Ok(())
    }
}
