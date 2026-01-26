use anyhow::Result;
use joule_profiler_cli::{CliArgs, output_format_to_displayer, parse_config};
use joule_profiler_core::JouleProfiler;
use joule_profiler_core::config::Config;
use source_rapl::Rapl;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = CliArgs::from_args()?;
    let displayer = output_format_to_displayer(&cli)?;
    let config: Config = parse_config(cli)?;

    let rapl = Rapl::try_from(&config)?;
    let mut profiler = JouleProfiler::default();
    profiler.set_displayer(displayer);
    profiler.add_source(rapl);

    match config.command {
        joule_profiler_core::config::Command::Profile(profile_config) => {
            profiler.run_phases(profile_config).await?;
        }
        joule_profiler_core::config::Command::ListSensors => {
            profiler.run_list_sensors()?;
        }
    }

    Ok(())
}
