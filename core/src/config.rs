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
//!     sockets: None,
//!     rapl_polling: Some(0.5),
//!     token_pattern: "__[A-Z0-9_]+__".into(),
//! };
//!
//! let config = Config {
//!     command: Command::Profile(profile),
//!     rapl_path: None,
//! };
//! ```

use std::collections::HashSet;

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

/// Configuration for profiling a program.
#[derive(Debug, Clone)]
pub struct ProfileConfig {
    /// Number of iterations (>= 1).
    pub iterations: usize,

    /// Optional file to redirect the profiled program stdout.
    pub stdout_file: Option<String>,

    /// Command and arguments to execute.
    pub cmd: Vec<String>,

    /// Optional set of CPU sockets to monitor.
    pub sockets: Option<HashSet<u32>>,

    /// Optional RAPL polling interval in seconds.
    pub rapl_polling: Option<f64>,

    /// Regex used to detect phase tokens in program output.
    pub token_pattern: String,
}
