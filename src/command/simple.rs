use anyhow::Result;
use log::{debug, info};

use crate::{
    command::run_command,
    config::SimpleConfig,
    measurement::MeasurementResult,
    output::{Displayer, OutputFormatTrait},
    source::{Metric, SourceManager, rapl::init_rapl},
    util::time::get_timestamp,
};

pub fn run_simple(config: &SimpleConfig) -> Result<()> {
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
        manager.start_workers();
        results.push(measure_simple(&mut manager, &config.cmd)?);
    }

    let mut displayer = Displayer::try_from(config)?;
    if config.iterations > 1 {
        displayer.simple_iterations(config, &results)?;
    } else {
        displayer.simple_single(config, &results[0])?;
    }
    Ok(())
}

fn measure_simple(manager: &mut SourceManager, command: &[String]) -> Result<MeasurementResult> {
    manager.start()?;

    let begin_time = get_timestamp();

    manager.measure()?;

    let (exit_code, _) = run_command(command, None)?;

    manager.measure()?;

    let end_time = get_timestamp();

    let result = manager.join()?;

    let mut metrics: Vec<Metric> = result.measures.into_iter().flatten().collect();
    metrics.sort_by_key(|metric| metric.name.clone());

    Ok(MeasurementResult {
        exit_code,
        duration_ms: end_time - begin_time,
        measure_count: result.count,
        metrics,
    })
}
