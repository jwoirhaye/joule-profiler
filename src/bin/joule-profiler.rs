use anyhow::Result;
use joule_profiler::JouleProfiler;
use joule_profiler::config::Config;

#[tokio::main]
async fn main() -> Result<()> {
    let config: Config = joule_profiler::cli::parse_config()?;
    JouleProfiler::try_from(config)?.run().await?;
    Ok(())
}
