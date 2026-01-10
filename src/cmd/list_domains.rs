use anyhow::Result;

use crate::{cli::ListArgs, config::OutputFormat, source::MetricSource};

pub fn run_list_sources(args: ListArgs, sources: &[MetricSource]) -> Result<()> {
    let output_format = OutputFormat::new(args.json, args.csv);

    for source in sources {
        source.print_source(output_format)?;
    }

    Ok(())
}
