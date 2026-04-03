//! Configuration for Joule Profiler.
//!
//! This module defines the configuration structures used to run the profiler:
//! which command to execute, how many iterations, output options, and RAPL settings.
//!
//! # Example
//!
//! ```no_run
//! use joule_profiler_core::config::{Config, Command, ProfileConfig};
//!
//! let profile = ProfileConfig {
//!     iterations: 1,
//!     stdout_file: None,
//!     cmd: vec!["sleep".into(), "1".into()],
//!     token_pattern: "__[A-Z0-9_]+__".into(),
//! };
//!
//! let config = Config {
//!     command: Command::Profile(profile),
//!     rapl_path: None,
//! };
//! ```

use derive_builder::Builder;

const PHASE_TOKEN_DEFAULT_REGEX_PATTERN: &str = "__[A-Z0-9_]+__";

/// Top-level configuration for Joule Profiler.
#[derive(Debug)]
pub struct Config {
    /// Action to run (profile a program or list sensors).
    pub command: Command,

    /// Override the base path used to read Intel RAPL counters.
    pub rapl_path: Option<String>,
}

/// Command executed by the profiler.
#[derive(Debug, Clone)]
pub enum Command {
    /// Run a program and collect metrics.
    Profile(ProfileConfig),

    /// List available sensors.
    ListSensors,
}

/// Configuration for program profiling.
#[derive(Debug, Clone, Builder)]
pub struct ProfileConfig {
    /// Optional file to redirect the profiled program stdout.
    #[builder(default, setter(strip_option))]
    pub stdout_file: Option<String>,

    /// Command and arguments to execute.
    pub cmd: Vec<String>,

    /// Regex used to detect phase tokens in program output.
    #[builder(default = PHASE_TOKEN_DEFAULT_REGEX_PATTERN.to_string())]
    pub token_pattern: String,
}
