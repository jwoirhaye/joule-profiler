use anyhow::Result;
use log::{debug, info, trace, warn};

use crate::config::Config;
use crate::measure::{MeasurementResult, PhasesResult};

use super::OutputFormat;

#[derive(Debug, Clone, Default)]
pub struct TerminalOutput;

impl TerminalOutput {
    pub fn new() -> Self {
        debug!("Terminal output formatter initialized");
        TerminalOutput
    }

    fn display_result(&self, res: &MeasurementResult, prefix: &str) -> Result<()> {
        trace!("Displaying measurement result with prefix: '{}'", prefix);

        println!();
        println!("{}═══════════════════════════════════════", prefix);
        println!("{}  Energy consumption (Joules)", prefix);
        println!("{}═══════════════════════════════════════", prefix);

        let mut keys: Vec<_> = res.energy_uj.keys().cloned().collect();
        keys.sort();

        let mut total_uj: u64 = 0;

        for k in &keys {
            if let Some(&v_uj) = res.energy_uj.get(k) {
                let v_j = (v_uj as f64) / 1_000_000.0;
                println!("{}  {:<20}: {:10.6} J", prefix, k, v_j);
                total_uj += v_uj;
            }
        }

        let duration_s = (res.duration_ms as f64) / 1000.0;
        let total_j = (total_uj as f64) / 1_000_000.0;
        let avg_power_w = if duration_s > 0.0 {
            total_j / duration_s
        } else {
            0.0
        };

        println!("{}───────────────────────────────────────", prefix);
        println!("{}  Total energy (J): {:10.6}", prefix, total_j);
        println!("{}  Average power (W): {:10.3}", prefix, avg_power_w);
        println!("{}  Duration (s)    : {:10.3}", prefix, duration_s);
        println!("{}  Exit code       : {}", prefix, res.exit_code);
        println!("{}═══════════════════════════════════════", prefix);

        trace!(
            "Displayed {} domain(s), total: {:.3} J, duration: {:.3} s",
            keys.len(),
            total_j,
            duration_s
        );

        Ok(())
    }

    fn display_statistics(&self, results: &[(usize, MeasurementResult)]) {
        if results.is_empty() {
            return;
        }

        info!("Computing statistics for {} iterations", results.len());

        let mut all_keys = std::collections::HashSet::new();
        for (_, res) in results {
            for key in res.energy_uj.keys() {
                all_keys.insert(key.clone());
            }
        }
        let mut keys: Vec<_> = all_keys.into_iter().collect();
        keys.sort();

        println!();
        println!("═══════════════════════════════════════");
        println!("  Statistics across {} iterations", results.len());
        println!("═══════════════════════════════════════");

        for key in &keys {
            let values: Vec<f64> = results
                .iter()
                .filter_map(|(_, res)| res.energy_uj.get(key))
                .map(|&uj| (uj as f64) / 1_000_000.0)
                .collect();

            if values.is_empty() {
                continue;
            }

            let mean = values.iter().sum::<f64>() / values.len() as f64;
            let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

            let mut sorted = values.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let median = if sorted.len().is_multiple_of(2) {
                (sorted[sorted.len() / 2 - 1] + sorted[sorted.len() / 2]) / 2.0
            } else {
                sorted[sorted.len() / 2]
            };

            let variance =
                values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
            let std_dev = variance.sqrt();

            println!("\n  Domain: {}", key);
            println!("    Mean   : {:10.6} J", mean);
            println!("    Median : {:10.6} J", median);
            println!("    Std Dev: {:10.6} J", std_dev);
            println!("    Min    : {:10.6} J", min);
            println!("    Max    : {:10.6} J", max);

            trace!(
                "Stats for {}: mean={:.6}, std={:.6}, range=[{:.6}, {:.6}]",
                key, mean, std_dev, min, max
            );
        }

        let durations: Vec<f64> = results
            .iter()
            .map(|(_, res)| (res.duration_ms as f64) / 1000.0)
            .collect();

        let mean_duration = durations.iter().sum::<f64>() / durations.len() as f64;
        let min_duration = durations.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max_duration = durations.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        println!("\n  Duration (s):");
        println!("    Mean   : {:10.3} s", mean_duration);
        println!("    Min    : {:10.3} s", min_duration);
        println!("    Max    : {:10.3} s", max_duration);

        println!("═══════════════════════════════════════");
    }
}

impl OutputFormat for TerminalOutput {
    fn simple_single(&mut self, res: &MeasurementResult) -> Result<()> {
        debug!("Formatting simple single measurement for terminal");
        self.display_result(res, "")
    }

    fn simple_iterations(
        &mut self,
        _config: &Config,
        results: &[(usize, MeasurementResult)],
    ) -> Result<()> {
        info!(
            "Formatting {} simple iterations for terminal",
            results.len()
        );

        for (idx, res) in results {
            println!("\n╔═══════════════════════════════════════╗");
            println!(
                "║  Iteration {} / {}                     ",
                idx + 1,
                results.len()
            );
            println!("╚═══════════════════════════════════════╝");
            self.display_result(res, "")?;
        }

        self.display_statistics(results);

        Ok(())
    }

    fn phases_single(&mut self, _config: &Config, phases: &PhasesResult) -> Result<()> {
        debug!(
            "Formatting phases single measurement for terminal ({} phases)",
            phases.phases.len()
        );

        for (i, phase) in phases.phases.iter().enumerate() {
            trace!(
                "Displaying phase {} / {}: {}",
                i + 1,
                phases.phases.len(),
                phase.name
            );

            println!();
            println!("╔═══════════════════════════════════════╗");
            println!("║  Phase: {:<30} ║", phase.name);
            println!("╚═══════════════════════════════════════╝");
            self.display_result(&phase.result, "")?;
        }

        Ok(())
    }

    fn phases_iterations(
        &mut self,
        _config: &Config,
        results: &[(usize, PhasesResult)],
    ) -> Result<()> {
        info!("Formatting {} phase iterations for terminal", results.len());

        if results.is_empty() {
            warn!("No phase iterations to display");
            return Ok(());
        }

        for (idx, phases_result) in results {
            println!();
            println!("╔═══════════════════════════════════════╗");
            println!(
                "║  Iteration {} / {}                     ",
                idx + 1,
                results.len()
            );
            println!("╚═══════════════════════════════════════╝");

            for phase in &phases_result.phases {
                trace!("Displaying phase '{}' for iteration {}", phase.name, idx);

                println!();
                println!("  ┌─────────────────────────────────────┐");
                println!("  │ Phase: {:<30}│", phase.name);
                println!("  └─────────────────────────────────────┘");
                self.display_result(&phase.result, "  ")?;
            }
        }

        if let Some((_, first_result)) = results.first() {
            println!();
            println!("╔═══════════════════════════════════════╗");
            println!("║  Statistics across {} iterations      ", results.len());
            println!("╚═══════════════════════════════════════╝");

            for phase_idx in 0..first_result.phases.len() {
                let phase_name = &first_result.phases[phase_idx].name;

                println!();
                println!("  Phase: {}", phase_name);
                println!("  ─────────────────────────────────────");

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
