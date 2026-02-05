use anyhow::Result;
use joule_profiler_cli::{
    CliArgs, ProfilerCommand, init_logging, output_format_to_displayer, parse_config,
};
use joule_profiler_core::JouleProfiler;
use joule_profiler_core::config::{Command, Config};
use log::warn;
use source_nvml::Nvml;
use source_rapl::Rapl;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = CliArgs::from_args()?;
    init_logging(cli.verbose);
    let mut displayer = output_format_to_displayer(&cli)?;
    let mut profiler = JouleProfiler::new();

    let rapl_polling = match &cli.command {
        ProfilerCommand::Phases(phases_args) => phases_args.rapl_polling,
        ProfilerCommand::ListSensors => None,
    };

    let rapl = Rapl::new(
        cli.rapl_path.as_deref(),
        cli.sockets.as_deref(),
        rapl_polling,
    )?;

    profiler.add_source(rapl);

    if cli.gpu {
        match Nvml::new() {
            Ok(nvml) => profiler.add_source(nvml),
            Err(err) => warn!("Failed to initialize NVML | {}", err),
        }
    }

    let config: Config = parse_config(cli)?;

    match config.command {
        Command::Profile(profile_config) => {
            let results = profiler.profile(&profile_config).await?;
            displayer.profile(&profile_config.cmd, &profile_config.token_pattern, &results)?;
        }
        Command::ListSensors => {
            let sensors = profiler.list_sensors()?;
            displayer.list_sensors(&sensors)?;
        }
    }

    Ok(())
}
