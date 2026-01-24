use clap::{ArgAction, Parser};

pub use commands::ProfilerCommand;

use crate::JouleProfilerError;

mod commands;

/// joule-profiler: measure program energy consumption
#[derive(Parser, Debug)]
#[command(name = "joule-profiler")]
#[command(
    version,
    about = "Measure program metrics from various sources like RAPL"
)]
pub struct Cli {
    /// Verbosity (-v, -vv, -vvv)
    #[arg(short = 'v', long = "verbose", action = ArgAction::Count)]
    pub verbose: u8,

    /// Override the base path used to read Intel RAPL counters.
    ///
    /// By default, the profiler reads from:
    ///   /sys/devices/virtual/powercap/intel-rapl
    ///
    /// If not provided, the profiler uses (by priority):
    ///   1. $JOULE_PROFILER_RAPL_PATH (if set)
    ///   2. /sys/devices/virtual/powercap/intel-rapl
    #[arg(long = "rapl-path")]
    pub rapl_path: Option<String>,

    /// Sockets to measure (e.g. 0 or 0,1)
    #[arg(short = 's', long = "sockets")]
    pub sockets: Option<String>,

    /// Export results as JSON instead of pretty terminal output
    #[arg(long, conflicts_with = "csv")]
    pub json: bool,

    /// Export results as CSV (semicolon-separated values)
    #[arg(long, conflicts_with = "json")]
    pub csv: bool,

    /// Output file for CSV/JSON (else `data<TIMESTAMP>`.csv/json)
    #[arg(short = 'o', long = "output-file")]
    pub output_file: Option<String>,

    /// The command to execute
    #[command(subcommand)]
    pub command: ProfilerCommand,
}

impl Cli {
    pub fn from_args() -> Result<Cli, JouleProfilerError> {
        let cli = Cli::try_parse()?;
        Ok(cli)
    }
}
