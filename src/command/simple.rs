use anyhow::Result;
use log::{debug, info};

use crate::{
    command::run_command,
    config::ProfileConfig,
    measurement::MeasurementResult,
    output::{Displayer, OutputFormatTrait},
    source::{Metric, SourceManager, rapl::init_rapl},
    util::time::get_timestamp,
};

pub async fn run_simple(config: &ProfileConfig) -> Result<()> {
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

    let mut displayer = Displayer::try_from(config)?;
    if config.iterations > 1 {
        displayer.simple_iterations(config, &results)?;
    } else {
        displayer.simple_single(config, &results[0])?;
    }
    Ok(())
}

async fn measure_simple(
    manager: &mut SourceManager,
    config: &ProfileConfig,
) -> Result<MeasurementResult> {
    manager.start().await?;

    let begin_time = get_timestamp();

    manager.measure().await?;

    let (exit_code, _) = run_command(&config.cmd, config.output_file.as_ref())?;

    manager.measure().await?;

    let end_time = get_timestamp();

    let result = manager.join().await?;

    let mut metrics: Vec<Metric> = result.measures.into_iter().flatten().collect();
    metrics.sort_by_key(|metric| metric.name.clone());
    let duration_ms = (end_time - begin_time) / 1000;

    Ok(MeasurementResult {
        exit_code,
        duration_ms,
        measure_count: result.count,
        metrics,
        measure_delta: result.measure_delta,
    })
}
