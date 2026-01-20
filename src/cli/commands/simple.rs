use super::CommonArgs;
use clap::Parser;

/// Arguments for Simple mode
#[derive(Parser, Debug)]
pub struct SimpleArgs {
    /// The common arguments between profiler commands
    #[command(flatten)]
    pub common: CommonArgs,
}
