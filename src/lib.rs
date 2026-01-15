use anyhow::Result;
use clap::Parser;
use env_logger::Builder;
use log::{LevelFilter, debug, info, trace};

use crate::{
    cli::Cli,
    command::{list_sensors::run_list_sensors, phases::run_phases, simple::run_simple},
    config::{Command, Config, ProfileConfig},
};

pub mod cli;
mod command;
mod config;
pub mod error;
mod measurement;
mod output;
pub mod source;
mod util;

/// Initialize and run Joule Profiler.
pub async fn run() -> Result<()> {
    let cli = Cli::try_parse()?;
    init_logging(cli.verbose);

    let config = Config::from(cli);

    info!("Joule Profiler starting");
    JouleProfiler::run(&config).await
}

pub struct JouleProfiler;

impl JouleProfiler {
    /// Run Joule Profiler.
    pub async fn run(config: &Config) -> Result<()> {
        match &config.mode {
            Command::Profile(profile_config) => Self::profile(profile_config).await,
            Command::ListSensors(list_config) => run_list_sensors(list_config),
        }
    }

    pub async fn profile(config: &ProfileConfig) -> Result<()> {
        match &config.mode {
            config::Mode::SimpleMode => run_simple(config).await,
            config::Mode::PhaseMode(phases_config) => run_phases(config, phases_config).await,
        }
    }
}

/// Initializes the logging system based on verbosity flags.
pub fn init_logging(level: u8) {
    let level_filter = match level {
        0 => LevelFilter::Warn,
        1 => LevelFilter::Info,
        2 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };

    Builder::new().filter_level(level_filter).init();

    match level {
        0 => {}
        1 => info!("Logging initialized at INFO level"),
        2 => debug!("Logging initialized at DEBUG level"),
        _ => trace!("Logging initialized at TRACE level"),
    }
}
