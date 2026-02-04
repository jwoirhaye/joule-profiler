use anyhow::Result;
use joule_profiler_cli::{CliArgs, ProfilerCommand, output_format_to_displayer, parse_config};
use joule_profiler_core::JouleProfiler;
use joule_profiler_core::config::{Command, Config};
use source_rapl::Rapl;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = CliArgs::from_args()?;
    let mut displayer = output_format_to_displayer(&cli)?;

    let rapl_polling = match &cli.command {
        ProfilerCommand::Phases(phases_args) => phases_args.rapl_polling,
        ProfilerCommand::ListSensors => None,
    };

    let rapl = Rapl::new(
        cli.rapl_path.as_deref(),
        cli.sockets.as_deref(),
        rapl_polling,
    )?;

    let mut profiler = JouleProfiler::new();
    profiler.add_source(rapl);

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
