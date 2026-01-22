use anyhow::Result;
use joule_profiler::JouleProfiler;

#[tokio::main]
async fn main() -> Result<()> {
    JouleProfiler::from_cli()?.run().await?;
    Ok(())
}
