use std::collections::HashSet;

use crate::{
    cli::{Cli, ProfilerCommand},
    output::{OutputFormat, output_format},
};

/// Configuration for running the profiler
#[derive(Debug)]
pub struct Config {
    /// The selected mode of operation (profiling or listing sensors)
    pub mode: Command,

    /// Optional path to the RAPL sysfs interface
    pub rapl_path: Option<String>,

    /// Format of the output (Terminal, JSON, CSV)
    pub output_format: OutputFormat,

    /// Optional file to write the output to
    pub output_file: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        let mode = Command::ListSensors(ListSensorsConfig {
            output_format: OutputFormat::Terminal,
        });
        Self {
            mode,
            rapl_path: Default::default(),
            output_format: Default::default(),
            output_file: Default::default(),
        }
    }
}

impl From<Cli> for Config {
    fn from(cli: Cli) -> Self {
        let sockets = cli.sockets.map(|s| {
            s.split(',')
                .filter_map(|x| x.trim().parse::<u32>().ok())
                .collect()
        });

        let mode = match cli.command {
            ProfilerCommand::Simple(simple) => {
                let common = simple.common;
                Command::Profile(ProfileConfig {
                    iterations: common.iterations.unwrap_or(1),
                    stdout_file: common.stdout_file,
                    cmd: common.cmd,
                    rapl_polling: common.rapl_polling,
                    mode: Mode::SimpleMode,
                    sockets,
                })
            }
            ProfilerCommand::Phases(phases) => {
                let common = phases.common;
                Command::Profile(ProfileConfig {
                    iterations: common.iterations.unwrap_or(1),
                    stdout_file: common.stdout_file,
                    cmd: common.cmd,
                    rapl_polling: common.rapl_polling,
                    mode: Mode::PhaseMode(PhasesConfig {
                        token_pattern: phases.token_pattern,
                    }),
                    sockets,
                })
            }

            ProfilerCommand::ListSensors(list) => Command::ListSensors(ListSensorsConfig {
                output_format: output_format(list.json, list.csv),
            }),
        };

        Config {
            mode,
            rapl_path: cli.rapl_path,
            output_format: output_format(cli.json, cli.csv),
            output_file: cli.output_file,
        }
    }
}

/// The configuration that can be executed
#[derive(Debug, Clone)]
pub enum Command {
    /// Profile a command with either simple or phase mode
    Profile(ProfileConfig),

    /// List available sensors in a given output format
    ListSensors(ListSensorsConfig),
}

/// Profiling configuration for a command
#[derive(Debug, Clone)]
pub struct ProfileConfig {
    /// Number of iterations to run the command
    pub iterations: usize,

    /// Optional file to redirect stdout
    pub stdout_file: Option<String>,

    /// Command and arguments to profile
    pub cmd: Vec<String>,

    /// Optional set of CPU sockets to monitor
    pub sockets: Option<HashSet<u32>>,

    /// Optional RAPL polling interval in seconds
    pub rapl_polling: Option<f64>,

    /// Profiling mode (simple or phases)
    pub mode: Mode,
}

/// Mode of profiling
#[derive(Debug, Clone)]
pub enum Mode {
    /// Simple profiling mode
    SimpleMode,

    /// Phase-based profiling with token extraction
    PhaseMode(PhasesConfig),
}

/// Phase-based profiling configuration
#[derive(Debug, Clone)]
pub struct PhasesConfig {
    /// Regex pattern to detect start and end tokens in command output
    pub token_pattern: String,
}

/// Configuration for listing sensors
#[derive(Debug, Clone)]
pub struct ListSensorsConfig {
    /// Output format for the sensor list
    pub output_format: OutputFormat,
}
