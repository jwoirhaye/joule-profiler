use clap::{Parser, Subcommand};

use crate::cli::commands::{list_sensors::ListSensorsArgs, phases::PhasesArgs};

pub mod list_sensors;
pub mod phases;

/// Subcommands of joule-profiler
#[derive(Subcommand, Debug)]
pub enum ProfilerCommand {
    /// Phase-based measurement mode (with start/end tokens).
    Phases(PhasesArgs),

    /// List available sensors.
    ListSensors(ListSensorsArgs),
}

/// Fields common to both Simple and Phases modes.
#[derive(Parser, Debug)]
pub struct CommonArgs {
    /// Number of iterations (>=1).
    #[arg(short = 'n', long = "iterations")]
    pub iterations: Option<usize>,

    /// Redirect profiled program stdout to this file.
    #[arg(short = 'o', long = "stdout-file")]
    pub stdout_file: Option<String>,

    /// Command to execute (everything after `--`).
    #[arg(last = true)]
    pub cmd: Vec<String>,

    /// Rapl polling frequency in second.
    #[arg(long = "rapl-polling")]
    pub rapl_polling: Option<f64>,
}
