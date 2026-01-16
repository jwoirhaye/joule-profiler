use super::CommonArgs;
use clap::Parser;

/// Arguments for Simple mode
#[derive(Parser, Debug)]
pub struct SimpleArgs {
    #[command(flatten)]
    pub common: CommonArgs,
}
