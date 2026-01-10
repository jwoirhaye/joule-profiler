use anyhow::Result;
use log::{debug, info, trace, warn};

use crate::{
    config::Config,
    measure::{MeasurementResult, PhasesResult},
    source::metric::Metric,
};

use super::OutputFormat;

/// Constants for formatting
const BORDER_DOUBLE: &str = "═";
const BORDER_SINGLE: &str = "─";
const BOX_WIDTH: usize = 50;

#[derive(Debug, Clone, Default)]
pub struct TerminalOutput;

impl TerminalOutput {
    pub fn new() -> Self {
        debug!("Terminal output formatter initialized");
        Self
    }

    /// Display command header
    fn display_command(&self, config: &Config) {
        if !config.cmd.is_empty() {
            println!();
            self.print_header("Command");
            println!("  {}", config.cmd.join(" "));
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
        trace!("Displaying measurement result with prefix: '{}'", prefix);

        println!();
        println!("{}{}", prefix, BORDER_DOUBLE.repeat(BOX_WIDTH));
        // println!("{}  Metrics", prefix);
        // println!("{}{}", prefix, BORDER_DOUBLE.repeat(BOX_WIDTH));

        let mut keys: Vec<_> = metrics.iter().map(|metric| &metric.name).cloned().collect();
        keys.sort_unstable();

        for metric in metrics {
            println!(
                "{}  {:<20}: {:10.6} {}",
                prefix, metric.name, metric.value, metric.unit
            );
        }

        // let duration_s = Self::ms_to_s(res.duration_ms);

        // println!("{}  {:<20}: {:>10.6} s", prefix, "Duration", duration_s);
        // println!("{}  {:<20}: {:>10}", prefix, "Exit code", res.exit_code);
        println!("{}{}", prefix, BORDER_DOUBLE.repeat(BOX_WIDTH));

        // trace!(
        //     "Displayed {} metrics, duration: {:.3} s",
        //     res.metrics.len(),
        //     duration_s
        // );

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

impl OutputFormat for TerminalOutput {
    fn simple_single(&mut self, config: &Config, res: &MeasurementResult) -> Result<()> {
        debug!("Formatting simple single measurement for terminal");
        self.display_command(config);
        self.display_result(&res.metrics, "")
    }

    fn simple_iterations(
        &mut self,
        config: &Config,
        results: &[(usize, MeasurementResult)],
    ) -> Result<()> {
        info!(
            "Formatting {} simple iterations for terminal",
            results.len()
        );

        self.display_command(config);

        for (idx, res) in results {
            self.display_iteration_header(*idx, results.len());
            self.display_result(&res.metrics, "")?;
        }

        Ok(())
    }

    fn phases_single(&mut self, config: &Config, phases: &PhasesResult) -> Result<()> {
        debug!(
            "Formatting phases single measurement for terminal ({} phases)",
            phases.phases.len()
        );

        self.display_command(config);

        for (i, phase) in phases.phases.iter().enumerate() {
            trace!(
                "Displaying phase {} / {}: {}",
                i + 1,
                phases.phases.len(),
                phase.name
            );
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
        config: &Config,
        results: &[(usize, PhasesResult)],
    ) -> Result<()> {
        info!("Formatting {} phase iterations for terminal", results.len());

        if results.is_empty() {
            warn!("No phase iterations to display");
            return Ok(());
        }

        self.display_command(config);

        for (idx, iteration_results) in results {
            self.display_iteration_header(*idx, results.len());

            for phase in &iteration_results.phases {
                trace!("Displaying phase '{}' for iteration {}", phase.name, idx);
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
}
