use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    joule_profiler::run().await
}
