use anyhow::Result;

use crate::{
    cli::ListArgs,
    config::OutputFormat,
    source::{SourceManager},
};

pub fn run_list_sensors(args: ListArgs, source_manager: &SourceManager) -> Result<()> {
    let output_format = OutputFormat::new(args.json, args.csv);

    source_manager.print_available_sensors(output_format)?;

    Ok(())
}
