use anyhow::Result;
use joule_profiler_cli::parse_config;
use joule_profiler_core::JouleProfiler;
use joule_profiler_core::config::Config;
use source_rapl::Rapl;

#[tokio::main]
async fn main() -> Result<()> {
    let config: Config = parse_config()?;

    let rapl = Rapl::try_from(&config)?;

    JouleProfiler::try_from((config, vec![rapl.into()]))?
        .run()
        .await?;

    Ok(())
}
