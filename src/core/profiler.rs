use anyhow::Result;
use log::{debug, info};
use tokio::time::Instant;

use crate::{
    commands::{list_sensors::run_list_sensors, phases::measure_phases, run_command},
    config::{
        Command, Config,
        profile::{Mode, PhasesConfig, ProfileConfig},
    },
    core::{
        displayer::ProfilerDisplayer, manager::SourceManager, measurement::MeasurementResult,
        metric::Metric,
    },
    sources::rapl::init_rapl,
};

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
            Mode::SimpleMode => Self::run_simple(config).await,
            Mode::PhaseMode(phases_config) => Self::run_phases(config, phases_config).await,
        }
    }

    async fn run_simple(config: &ProfileConfig) -> Result<()> {
        info!("Running simple mode");

        let sources = vec![init_rapl(
            config.rapl_path.as_deref(),
            config.sockets.as_ref(),
            config.rapl_polling,
        )?];
        let mut manager = SourceManager::new(sources);

        let mut results = Vec::new();

        debug!("Simple mode with {} iteration(s)", config.iterations);
        for _ in 0..config.iterations {
            manager.start_workers().await;
            results.push(measure_simple(&mut manager, config).await?);
        }

        let mut displayer: Box<dyn ProfilerDisplayer> = config.try_into()?;

        if config.iterations > 1 {
            displayer.simple_iterations(&config.cmd, &results)?;
        } else {
            displayer.simple_single(&config.cmd, &results[0])?;
        }
        Ok(())
    }

    pub async fn run_phases(config: &ProfileConfig, phases_config: &PhasesConfig) -> Result<()> {
        let sources = vec![init_rapl(
            config.rapl_path.as_deref(),
            config.sockets.as_ref(),
            config.rapl_polling,
        )?];
        let mut manager = SourceManager::new(sources);

        let mut results = Vec::new();

        for _ in 0..config.iterations {
            manager.start_workers().await;
            results.push(measure_phases(&mut manager, config, phases_config).await?);
        }

        let mut displayer: Box<dyn ProfilerDisplayer> = config.try_into()?;

        if config.iterations > 1 {
            displayer.phases_iterations(&config.cmd, &phases_config.token_pattern, &results)?;
        } else {
            displayer.phases_single(&config.cmd, &phases_config.token_pattern, &results[0])?;
        }

        Ok(())
    }
}

async fn measure_simple(
    manager: &mut SourceManager,
    config: &ProfileConfig,
) -> Result<MeasurementResult> {
    manager.start().await?;
    manager.measure().await?;

    let begin_instant = Instant::now();

    let (exit_code, _) = run_command(&config.cmd, config.output_file.as_ref())?;

    let end_instant = Instant::now();

    manager.measure().await?;
    manager.stop().await?;

    let result = manager.retrieve().await?;

    let mut metrics: Vec<Metric> = result.measures.into_iter().flatten().collect();
    metrics.sort_by_key(|metric| metric.name.clone());

    let duration_ms = (end_instant - begin_instant).as_millis();

    Ok(MeasurementResult {
        exit_code,
        duration_ms,
        measure_count: result.count,
        metrics,
        measure_delta: result.measure_delta,
    })
}
