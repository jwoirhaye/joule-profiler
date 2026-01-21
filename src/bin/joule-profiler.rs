use anyhow::Result;
use joule_profiler::{JouleProfiler, JouleProfilerError};

#[tokio::main]
async fn main() -> Result<(), JouleProfilerError> {
    JouleProfiler::from_cli()?.run().await
}
