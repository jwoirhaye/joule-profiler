use anyhow::Result;
use clap::Parser;
use log::info;

use crate::{cli::Cli, config::Config, util::logging::init_logging};

pub use core::profiler::JouleProfiler;

pub mod cli;
mod config;
mod core;
mod output;
mod sources;
mod util;

/// Initialize and run Joule Profiler.
pub async fn run() -> Result<()> {
    let cli = Cli::try_parse()?;
    init_logging(cli.verbose);

    let config = Config::from(cli);

    info!("Joule Profiler starting");
    let mut profiler = JouleProfiler::try_from(config)?;
    profiler.run().await?;

    Ok(())
}