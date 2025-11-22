use anyhow::Result;
use log::{debug, info};

use crate::cli::PhasesArgs;
use crate::config::{Config, OutputFormat};
use crate::measure::{measure_phases_iterations, measure_phases_once};
use crate::output::csv::CsvOutput;
use crate::output::{JsonOutput, OutputFormat as OutputFormatTrait, TerminalOutput};
use crate::rapl::RaplDomain;

pub fn run_phases(args: PhasesArgs, domains: &[RaplDomain]) -> Result<()> {
    info!("Running phases mode");
    let config = Config::from_phases(args, domains)?;

    if let Some(n) = config.iterations {
        debug!("Phases mode with {} iteration(s)", n);
        run_phases_iterations(&config, domains, n)
    } else {
        debug!("Phases mode with single measurement");
        run_phases_single(&config, domains)
    }
}

fn run_phases_single(config: &Config, domains: &[RaplDomain]) -> Result<()> {
    info!("Measuring single phases execution");
    let res = measure_phases_once(config, domains)?;

    debug!("Phases measurement complete, formatting output");

    match config.output_format() {
        OutputFormat::Json => {
            debug!("Using JSON output format (stdout)");
            let mut out = JsonOutput::new(config)?;
            out.phases_single(config, &res)?;
        }
        OutputFormat::Csv => {
            debug!("Using CSV output format (file)");
            let mut out = CsvOutput::new(config)?;
            out.phases_single(config, &res)?;
        }
        OutputFormat::Terminal => {
            debug!("Using terminal output format");
            let mut out = TerminalOutput::new();
            out.phases_single(config, &res)?;
        }
    }

    info!("Phases single measurement completed successfully");
    Ok(())
}

fn run_phases_iterations(config: &Config, domains: &[RaplDomain], iterations: usize) -> Result<()> {
    info!("Running {} iteration(s) in phases mode", iterations);
    let results = measure_phases_iterations(config, domains, iterations)?;

    debug!("All iterations complete, formatting output");

    match config.output_format() {
        OutputFormat::Json => {
            debug!("Using JSON output format (file)");
            let mut out = JsonOutput::new(config)?;
            out.phases_iterations(config, &results)?;
        }
        OutputFormat::Csv => {
            debug!("Using CSV output format (file)");
            let mut out = CsvOutput::new(config)?;
            out.phases_iterations(config, &results)?;
        }
        OutputFormat::Terminal => {
            debug!("Using terminal output format");
            let mut out = TerminalOutput::new();
            out.phases_iterations(config, &results)?;
        }
    }

    info!("Phases iterations completed successfully");
    Ok(())
}
