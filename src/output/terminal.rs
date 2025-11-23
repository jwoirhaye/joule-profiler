use anyhow::Result;
use log::{debug, info, trace, warn};
use std::collections::HashSet;

use crate::config::Config;
use crate::measure::{MeasurementResult, PhasesResult};

use super::OutputFormat;

/// Constants for formatting
const BORDER_DOUBLE: &str = "═";
const BORDER_SINGLE: &str = "─";
const BOX_WIDTH: usize = 50;

/// Statistics for a domain
#[derive(Debug)]
struct DomainStats {
    mean: f64,
    median: f64,
    std_dev: f64,
    min: f64,
    max: f64,
}

impl DomainStats {
    fn calculate(values: &[f64]) -> Option<Self> {
        if values.is_empty() {
            return None;
        }

        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let min = values.iter().copied().fold(f64::INFINITY, f64::min);
        let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);

        let mut sorted = values.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let median = if sorted.len().is_multiple_of(2) {
            (sorted[sorted.len() / 2 - 1] + sorted[sorted.len() / 2]) / 2.0
        } else {
            sorted[sorted.len() / 2]
        };

        let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
        let std_dev = variance.sqrt();

        Some(Self {
            mean,
            median,
            std_dev,
            min,
            max,
        })
    }
}

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

    /// Convert microjoules to joules
    #[inline]
    const fn uj_to_j(uj: u64) -> f64 {
        uj as f64 / 1_000_000.0
    }

    /// Convert milliseconds to seconds
    #[inline]
    const fn ms_to_s(ms: u128) -> f64 {
        ms as f64 / 1000.0
    }

    /// Display a single measurement result
    fn display_result(&self, res: &MeasurementResult, prefix: &str) -> Result<()> {
        trace!("Displaying measurement result with prefix: '{}'", prefix);

        println!();
        println!("{}{}", prefix, BORDER_DOUBLE.repeat(BOX_WIDTH));
        println!("{}  Energy consumption (Joules)", prefix);
        println!("{}{}", prefix, BORDER_DOUBLE.repeat(BOX_WIDTH));

        let mut keys: Vec<_> = res.energy_uj.keys().cloned().collect();
        keys.sort_unstable();

        let total_uj: u64 = keys.iter().filter_map(|k| res.energy_uj.get(k)).sum();

        for key in &keys {
            if let Some(&v_uj) = res.energy_uj.get(key) {
                let v_j = Self::uj_to_j(v_uj);
                println!("{}  {:<20}: {:10.6} J", prefix, key, v_j);
            }
        }

        let duration_s = Self::ms_to_s(res.duration_ms);
        let total_j = Self::uj_to_j(total_uj);
        let avg_power_w = if duration_s > 0.0 {
            total_j / duration_s
        } else {
            0.0
        };

        println!("{}{}", prefix, BORDER_SINGLE.repeat(BOX_WIDTH));
        println!("{}  {:<20}: {:>10.6} J", prefix, "Total energy", total_j);
        println!(
            "{}  {:<20}: {:>10.6} W",
            prefix, "Average power", avg_power_w
        );
        println!("{}  {:<20}: {:>10.6} s", prefix, "Duration", duration_s);
        println!("{}  {:<20}: {:>10}", prefix, "Exit code", res.exit_code);
        println!("{}{}", prefix, BORDER_DOUBLE.repeat(BOX_WIDTH));

        trace!(
            "Displayed {} domain(s), total: {:.3} J, duration: {:.3} s",
            keys.len(),
            total_j,
            duration_s
        );

        Ok(())
    }

    /// Extract all unique domain keys from results
    fn extract_domain_keys(results: &[(usize, MeasurementResult)]) -> Vec<String> {
        let mut all_keys = HashSet::new();
        for (_, res) in results {
            all_keys.extend(res.energy_uj.keys().cloned());
        }
        let mut keys: Vec<_> = all_keys.into_iter().collect();
        keys.sort_unstable();
        keys
    }

    /// Display statistics for a set of results
    fn display_statistics(&self, results: &[(usize, MeasurementResult)]) {
        if results.is_empty() {
            return;
        }

        info!("Computing statistics for {} iterations", results.len());

        let keys = Self::extract_domain_keys(results);

        println!();
        println!("{}", BORDER_DOUBLE.repeat(BOX_WIDTH));
        println!("  Statistics across {} iterations", results.len());
        println!("{}", BORDER_DOUBLE.repeat(BOX_WIDTH));

        for key in &keys {
            let values: Vec<f64> = results
                .iter()
                .filter_map(|(_, res)| res.energy_uj.get(key))
                .map(|&uj| Self::uj_to_j(uj))
                .collect();

            if let Some(stats) = DomainStats::calculate(&values) {
                self.display_domain_stats(key, &stats);
            }
        }

        self.display_duration_stats(results);
        println!("{}", BORDER_DOUBLE.repeat(BOX_WIDTH));
    }

    /// Display statistics for a single domain
    fn display_domain_stats(&self, domain: &str, stats: &DomainStats) {
        println!("\n  Domain: {}", domain);
        println!("    Mean   : {:10.6} J", stats.mean);
        println!("    Median : {:10.6} J", stats.median);
        println!("    Std Dev: {:10.6} J", stats.std_dev);
        println!("    Min    : {:10.6} J", stats.min);
        println!("    Max    : {:10.6} J", stats.max);

        trace!(
            "Stats for {}: mean={:.6}, std={:.6}, range=[{:.6}, {:.6}]",
            domain, stats.mean, stats.std_dev, stats.min, stats.max
        );
    }

    /// Display duration statistics
    fn display_duration_stats(&self, results: &[(usize, MeasurementResult)]) {
        let durations: Vec<f64> = results
            .iter()
            .map(|(_, res)| Self::ms_to_s(res.duration_ms))
            .collect();

        if let Some(stats) = DomainStats::calculate(&durations) {
            println!("\n  Duration (s):");
            println!("    Mean   : {:10.3} s", stats.mean);
            println!("    Min    : {:10.3} s", stats.min);
            println!("    Max    : {:10.3} s", stats.max);
        }
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

        // Add a blank line for spacing if tokens were displayed
        if start_token.is_some() || end_token.is_some() {
            println!();
        }
    }
}

impl OutputFormat for TerminalOutput {
    fn simple_single(&mut self, config: &Config, res: &MeasurementResult) -> Result<()> {
        debug!("Formatting simple single measurement for terminal");
        self.display_command(config);
        self.display_result(res, "")
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
            self.display_result(res, "")?;
        }

        self.display_statistics(results);
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
            self.display_result(&phase.result, "")?;
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

        for (idx, phases_result) in results {
            self.display_iteration_header(*idx, results.len());

            for phase in &phases_result.phases {
                trace!("Displaying phase '{}' for iteration {}", phase.name, idx);
                self.display_phase_header(
                    &phase.name,
                    phase.start_token.as_deref(),
                    phase.end_token.as_deref(),
                    phase.start_line,
                    phase.end_line,
                    "  ",
                );
                self.display_result(&phase.result, "  ")?;
            }
        }

        // Display statistics per phase
        if let Some((_, first_result)) = results.first() {
            println!();
            println!("╔{}╗", BORDER_DOUBLE.repeat(BOX_WIDTH));
            println!(
                "║  Statistics across {} iterations{:<width$} ║",
                results.len(),
                "",
                width = BOX_WIDTH - 34
            );
            println!("╚{}╝", BORDER_DOUBLE.repeat(BOX_WIDTH));

            for (phase_idx, phase) in first_result.phases.iter().enumerate() {
                println!();
                println!("  Phase: {}", phase.name);
                println!("  {}", BORDER_SINGLE.repeat(BOX_WIDTH - 2));

                let phase_results: Vec<(usize, MeasurementResult)> = results
                    .iter()
                    .filter_map(|(idx, pr)| {
                        pr.phases.get(phase_idx).map(|p| (*idx, p.result.clone()))
                    })
                    .collect();

                if !phase_results.is_empty() {
                    self.display_statistics(&phase_results);
                }
            }
        }

        Ok(())
    }
}