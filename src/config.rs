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

use crate::output::OutputFormat;

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
