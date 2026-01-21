use crate::core::{
    aggregate::Metric,
    displayer::{Displayer, Result},
    phase::PhaseToken,
    profiler::types::Iteration,
    sensor::Sensor,
};

/// Constants for formatting
const BORDER_DOUBLE: &str = "═";
const BORDER_SINGLE: &str = "─";
const BOX_WIDTH: usize = 50;

#[derive(Debug, Clone, Default)]
pub struct TerminalOutput;

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
        start_token: &PhaseToken,
        end_token: &PhaseToken,
        start_line: Option<usize>,
        prefix: &str,
    ) {
        let phase_name = &format!("{} -> {}", start_token, end_token);

        println!();
        if prefix.is_empty() {
            println!("╔{}╗", BORDER_DOUBLE.repeat(BOX_WIDTH));
            println!("║  Phase: {:<width$} ║", phase_name, width = BOX_WIDTH - 10);
            println!("╚{}╝", BORDER_DOUBLE.repeat(BOX_WIDTH));
        } else {
            self.print_subheader(&format!("Phase: {}", phase_name), prefix);
        }

        // Display token information
        let start_info = if let Some(line) = start_line {
            format!("{} (line {})", start_token, line)
        } else {
            start_token.to_string()
        };
        println!("{}  Start token: {}", prefix, start_info);
    }
}

impl Displayer for TerminalOutput {
    fn simple_single(&mut self, cmd: &[String], result: &Iteration) -> Result<()> {
        self.display_command(cmd);
        self.display_result(&result.phases[0].metrics, "")?;
        Ok(())
    }

    fn simple_iterations(&mut self, cmd: &[String], results: &[Iteration]) -> Result<()> {
        self.display_command(cmd);

        for (idx, result) in results.iter().enumerate() {
            self.display_iteration_header(idx, results.len());
            self.display_result(&result.phases[0].metrics, "")?;
        }

        Ok(())
    }

    fn phases_single(
        &mut self,
        cmd: &[String],
        _token_pattern: &str,
        iteration: &Iteration,
    ) -> Result<()> {
        self.display_command(cmd);

        for phase in &iteration.phases {
            self.display_phase_header(&phase.start_token, &phase.end_token, phase.line_number, "");
            self.display_result(&phase.metrics, "")?;
        }

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

        for (idx, iteration_results) in iterations.iter().enumerate() {
            self.display_iteration_header(idx, nb_iterations);

            for phase in &iteration_results.phases {
                self.display_phase_header(
                    &phase.start_token,
                    &phase.end_token,
                    phase.line_number,
                    "  ",
                );
                self.display_result(&phase.metrics, "  ")?;
            }
        }

        Ok(())
    }

    fn list_sensors(&mut self, sensors: &[Sensor]) -> Result<()> {
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
