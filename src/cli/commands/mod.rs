use clap::{Parser, Subcommand};

use crate::cli::commands::{list_sensors::ListSensorsArgs, phases::PhasesArgs, simple::SimpleArgs};

pub mod list_sensors;
pub mod phases;
pub mod simple;

/// Subcommands of joule-profiler
#[derive(Subcommand, Debug)]
pub enum ProfilerCommand {
    /// Standard measurement mode (single or repeated)
    Simple(SimpleArgs),

    /// Phase-based measurement mode (with start/end tokens)
    Phases(PhasesArgs),

    /// List available RAPL energy domains
    ListSensors(ListSensorsArgs),
}

/// Fields common to both Simple and Phases modes
#[derive(Parser, Debug)]
pub struct CommonArgs {
    /// Export results as JSON instead of pretty terminal output
    #[arg(long, conflicts_with = "csv")]
    pub json: bool,

    /// Export results as CSV (semicolon-separated values)
    #[arg(long, conflicts_with = "json")]
    pub csv: bool,

    /// Number of iterations (>=1)
    #[arg(short = 'n', long = "iterations")]
    pub iterations: Option<usize>,

    /// Output file for CSV/JSON (else data<TIMESTAMP>.csv/json)
    #[arg(long = "jouleit-file")]
    pub jouleit_file: Option<String>,

    /// Redirect profiled program stdout to this file
    #[arg(short = 'o', long = "output-file")]
    pub output_file: Option<String>,

    /// Command to execute (everything after `--`)
    #[arg(last = true)]
    pub cmd: Vec<String>,

    #[arg(long = "rapl-polling")]
    pub rapl_polling: Option<f64>,
}
