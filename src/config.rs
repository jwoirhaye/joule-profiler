//! Configuration module for Joule Profiler.
//!
//! This module defines all configuration structures used to setup the profiler,
//! including commands to run, profiling modes, output formats, and sensor selection.
//!
//! # Examples
//!
//! ```no_run
//! use joule_profiler::config::{Config, ProfileConfig, Mode, Command};
//!
//! let profile_config = ProfileConfig {
//!     iterations: 3,
//!     stdout_file: None,
//!     cmd: vec!["sleep".into(), "1".into()],
//!     sockets: None,
//!     rapl_polling: Some(0.5),
//!     mode: Mode::SimpleMode,
//! };
//!
//! let config = Config {
//!     command: Command::Profile(profile_config),
//!     rapl_path: None,
//!     output_format: Default::default(),
//!     output_file: None,
//! };
//! ```

use std::collections::HashSet;

use crate::{
    cli::{Cli, ProfilerCommand},
    output::{OutputFormat, output_format},
};

/// Top-level configuration for Joule Profiler.
///
/// # Fields
///
/// - `command` ([`Command`]) - Execution mode of the profiler (e.g., [`Mode::SimpleMode`] or Phase).
/// - `rapl_path` (`Option<String>`) - Path to RAPL domains. Defaults to the standard RAPL path if not provided.
/// - `output_format` ([`OutputFormat`]) - Format for outputting results (e.g., terminal, JSON, CSV).
/// - `output_file` (`Option<String>`) - File to store results, if any.
///
/// # Examples
///
/// ```no_run
/// use joule_profiler::{
///     config::{Config, ProfileConfig, Command, Mode},
///     output::OutputFormat
/// };
///
/// let profile_config = ProfileConfig {
///     iterations: 1,
///     stdout_file: None,
///     cmd: vec!["sleep".into(), "1".into()],
///     rapl_polling: None,
///     mode: Mode::SimpleMode,
///     sockets: None,
/// };
///
/// let s = Config {
///     command: Command::Profile(profile_config),
///     rapl_path: None,
///     output_format: OutputFormat::default(),
///     output_file: None,
/// };
/// ```
#[derive(Debug)]
pub struct Config {
    pub command: Command,
    pub rapl_path: Option<String>,
    pub output_format: OutputFormat,
    pub output_file: Option<String>,
}

/// Converts a [`Cli`] instance into a [`Config`] for the Joule Profiler.
///
/// This implementation allows constructing a `Config` from the parsed CLI
/// arguments. It maps the user's command-line options into the profiler's
/// internal configuration structure.
///
/// # Behavior
///
/// - Parses `cli.sockets` from a comma-separated string into a `Vec<u32>`.
/// - Maps the CLI command into the corresponding [`Command`] variant:
///     - [`ProfilerCommand::Simple`] into [`Command::Profile`] with [`Mode::SimpleMode`].
///     - [`ProfilerCommand::Phases`] into [`Command::Profile`] with [`Mode::PhaseMode`] containing
///       a [`PhasesConfig`].
///     - [`ProfilerCommand::ListSensors`] into [`Command::ListSensors`] with [`ListSensorsConfig`].
/// - Sets `rapl_path`, `output_format`, and `output_file` according to CLI options.
///
/// # Examples
///
/// ```no_run
/// use joule_profiler::{
///     config::Config,
///     cli::Cli,
/// };
///
/// let cli = Cli::from_args().expect("Failed to parse CLI arguments");
/// let config: Config = cli.into();
/// ```
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
            command: mode,
            rapl_path: cli.rapl_path,
            output_format: output_format(cli.json, cli.csv),
            output_file: cli.output_file,
        }
    }
}

/// Represents a command that the Joule Profiler can execute.
///
/// # Variants
///
/// - [`Command::Profile`] ([`ProfileConfig`]): Run a command in either simple or phase mode.
/// - [`Command::ListSensors`] ([`ListSensorsConfig`]): List available sensors in a given output format.
#[derive(Debug, Clone)]
pub enum Command {
    Profile(ProfileConfig),
    ListSensors(ListSensorsConfig),
}

/// Mode of profiling.
///
/// # Variants
///
/// - [`Mode::SimpleMode`]: Run the profiler in simple mode, measuring the whole command as one phase.
/// - [`Mode::PhaseMode`] ([`PhasesConfig`]): Run the profiler in phase mode, splitting the command output based on tokens.
#[derive(Debug, Clone)]
pub enum Mode {
    SimpleMode,
    PhaseMode(PhasesConfig),
}

/// Profiling configuration for a command.
///
/// # Fields
///
/// - `iterations` (`usize`): Number of iterations to run the command.
/// - `stdout_file` (`Option<String>`): Optional file to redirect stdout.
/// - `cmd` (`Vec<String>`): Command and arguments to profile.
/// - `sockets` (`Option<HashSet<u32>>`): Optional set of CPU sockets to monitor.
/// - `rapl_polling` (`Option<f64>`): Optional RAPL polling interval in seconds.
/// - `mode` ([`Mode`]): Profiling mode (simple or phases).
///
/// # Examples
///
/// ```no_run
/// use joule_profiler::config::{ProfileConfig, Mode};
///
/// let config = ProfileConfig {
///     iterations: 3,
///     stdout_file: None,
///     cmd: vec!["sleep".into(), "1".into()],
///     sockets: None,
///     rapl_polling: Some(0.5),
///     mode: Mode::SimpleMode,
/// };
/// ```
#[derive(Debug, Clone)]
pub struct ProfileConfig {
    pub iterations: usize,
    pub stdout_file: Option<String>,
    pub cmd: Vec<String>,
    pub sockets: Option<HashSet<u32>>,
    pub rapl_polling: Option<f64>,
    pub mode: Mode,
}

/// Phase-based profiling configuration.
///
/// # Fields
///
/// - `token_pattern`: Regex pattern to detect start and end tokens in command output.
///
/// # Example
///
/// ```
/// use joule_profiler::config::PhasesConfig;
///
/// let phases = PhasesConfig {
///     token_pattern: "[PHASE]".to_string(),
/// };
/// ```
#[derive(Debug, Clone)]
pub struct PhasesConfig {
    pub token_pattern: String,
}

/// Configuration for listing sensors.
///
/// # Fields
///
/// - `output_format` ([`OutputFormat`]): Output format for the sensor list.
#[derive(Debug, Clone)]
pub struct ListSensorsConfig {
    pub output_format: OutputFormat,
}
