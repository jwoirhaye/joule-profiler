use anyhow::Result;
use joule_profiler_cli::{
    CliArgs, ProfilerCommand, init_logging, output_format_to_displayer, parse_config,
};
use joule_profiler_core::JouleProfiler;
use joule_profiler_core::config::{Command, Config};
use log::{trace, warn};
use source_nvml::Nvml;
use source_rapl::error::RaplError;
use source_rapl::perf::PerfRapl;
use source_rapl::powercap::PowercapRapl;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = CliArgs::from_args()?;
    init_logging(cli.verbose);
    let mut displayer = output_format_to_displayer(&cli)?;
    let mut profiler = JouleProfiler::new();

    let rapl_path = cli.rapl_path.as_deref();
    let rapl_sockets = cli.sockets.as_deref();
    let rapl_polling = match &cli.command {
        ProfilerCommand::Phases(phases_args) => phases_args.rapl_polling,
        ProfilerCommand::ListSensors => None,
    };

    match cli.rapl_backend {
        joule_profiler_cli::RaplBackend::Perf => {
            if let Err(err) = PerfRapl::check_perf_access() {
                warn!(
                    "Cannot initialize RAPL with perf, switching to powercap: {}",
                    err
                );
                let rapl_powercap = init_powercap(rapl_path, rapl_sockets, rapl_polling)?;
                profiler.add_source(rapl_powercap);
            } else {
                trace!("Using perf_events for RAPL profiling");
                let perf_rapl = PerfRapl::new(rapl_path, rapl_sockets)?;
                profiler.add_source(perf_rapl);
            }
        }
        joule_profiler_cli::RaplBackend::Powercap => {
            trace!("Using Powercap for RAPL profiling");
            let rapl_powercap = init_powercap(rapl_path, rapl_sockets, rapl_polling)?;
            profiler.add_source(rapl_powercap);
        }
    }

    if cli.gpu {
        match Nvml::new() {
            Ok(nvml) => profiler.add_source(nvml),
            Err(err) => warn!("{}", err),
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

fn init_powercap(
    rapl_path: Option<&str>,
    rapl_sockets: Option<&str>,
    rapl_polling: Option<f64>,
) -> std::result::Result<PowercapRapl, RaplError> {
    PowercapRapl::new(rapl_path, rapl_sockets, rapl_polling)
}
