use anyhow::Result;
use joule_profiler_cli::output_format_to_displayer;
use joule_profiler_cli::config_table::ConfigTable;
use joule_profiler_cli::{CliArgs, register_sources};
use joule_profiler_core::JouleProfiler;
use joule_profiler_core::config::{Command, Config};
use source_nvml::Nvml;
use source_perf_event::PerfEvent;
use source_rapl::{perf, powercap};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = CliArgs::from_args();

    let mut profiler = JouleProfiler::new();

    let mut config_table = if let Some(config_file) = &cli.config_file {
        let content = std::fs::read_to_string(config_file)?;
        let mut value: toml::Table = toml::from_str(&content)?;
        let sources = value
            .remove("sources")
            .unwrap_or(toml::Value::Table(toml::Table::new()))
            .try_into()?;

        ConfigTable::new(sources, &cli)
    } else {
        ConfigTable::new(toml::Table::new(), &cli)
    };

    register_sources!(config_table, [Nvml, PerfEvent, powercap::Rapl, perf::Rapl]);

    let sources = config_table.build_sources()?;
    profiler.set_sources(sources);

    let mut displayer = output_format_to_displayer(&cli)?;
    let config = Config::from(cli);

    match config.command {
        Command::Profile(profile_config) => {
            let results = profiler.profile(&profile_config).await?;
            displayer.display_results(
                &profile_config.cmd,
                &profile_config.token_pattern,
                &results,
            )?;
        }
        Command::ListSensors => {
            let sensors = profiler.list_sensors()?;
            displayer.list_sensors(&sensors)?;
        }
    }

    Ok(())
}
