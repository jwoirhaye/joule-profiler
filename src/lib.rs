use anyhow::Result;
use clap::Parser;
use env_logger::Builder;
use log::{LevelFilter, debug, info, trace};

use crate::{
    cli::Cli,
    command::{list_sensors::run_list_sensors, phases::run_phases, simple::run_simple},
    config::{Command, Config},
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
pub fn run() -> Result<()> {
    let cli = Cli::try_parse()?;
    init_logging(cli.verbose);

    let config = Config::from(cli);

    info!("Joule Profiler starting");
    JouleProfiler::run(&config)
}

pub struct JouleProfiler;

impl JouleProfiler {
    /// Run Joule Profiler.
    pub fn run(config: &Config) -> Result<()> {
        match &config.mode {
            Command::Simple(simple_config) => run_simple(simple_config),
            Command::Phases(phases_config) => run_phases(phases_config),
            Command::ListSensors(list_sensors_config) => run_list_sensors(list_sensors_config),
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
