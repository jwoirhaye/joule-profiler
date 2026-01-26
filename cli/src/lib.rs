use std::collections::HashSet;

use clap::{ArgAction, Parser};

use anyhow::Result;
pub use commands::ProfilerCommand;
use displayer_csv::CsvOutput;
use displayer_json::JsonOutput;
use displayer_terminal::TerminalOutput;
use joule_profiler_core::{
    config::{Command, Config, ProfileConfig},
    displayer::Displayer,
};

use crate::output::{OutputFormat, output_format};
mod commands;
mod logging;
mod output;

/// joule-profiler: measure program energy consumption
#[derive(Parser, Debug)]
#[command(name = "joule-profiler")]
#[command(
    version,
    about = "Measure program metrics from various sources like RAPL"
)]
pub struct CliArgs {
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

impl CliArgs {
    pub fn from_args() -> Result<Self> {
        Ok(Self::try_parse()?)
    }
}

impl From<CliArgs> for Config {
    fn from(cli_args: CliArgs) -> Self {
        let sockets: Option<HashSet<u32>> = cli_args.sockets.map(|s| {
            s.split(',')
                .filter_map(|x| x.trim().parse::<u32>().ok())
                .collect()
        });

        let command = match cli_args.command {
            ProfilerCommand::Phases(phases) => Command::Profile(ProfileConfig {
                iterations: phases.iterations.unwrap_or(1),
                stdout_file: phases.stdout_file,
                cmd: phases.cmd,
                rapl_polling: phases.rapl_polling,
                token_pattern: phases.token_pattern,
                sockets,
            }),

            ProfilerCommand::ListSensors => Command::ListSensors,
        };

        Config {
            command,
            rapl_path: cli_args.rapl_path,
        }
    }
}

pub fn output_format_to_displayer(cli: &CliArgs) -> Result<Box<dyn Displayer>> {
    let output_format = output_format(cli.json, cli.csv);
    let output_file = cli.output_file.clone();

    let displayer = match output_format {
        OutputFormat::Terminal => TerminalOutput.into(),
        OutputFormat::Json => JsonOutput::new(output_file)?.into(),
        OutputFormat::Csv => CsvOutput::try_new(output_file)?.into(),
    };

    Ok(displayer)
}

pub fn parse_config(cli: CliArgs) -> Result<Config> {
    logging::init_logging(cli.verbose);
    Ok(cli.into())
}
