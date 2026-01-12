use anyhow::Result;

use crate::{
    cli::{Cli, Command},
    source::{
        MetricSource,
        rapl::{Rapl, get_domains},
    },
};

mod list_domains;
mod phases;
mod simple;

pub use list_domains::run_list_sources;
pub use phases::run_phases;
pub use simple::run_simple;

pub fn run(cli: Cli) -> Result<()> {
    let domains = get_domains(cli.rapl_path.as_deref(), cli.sockets.as_deref())?;

    let rapl = Rapl::new(domains)?;
    let mut sources: Vec<MetricSource> = vec![MetricSource::Rapl(rapl)];

    match cli.command {
        Command::Simple(args) => run_simple(args, &mut sources),
        Command::Phases(args) => run_phases(args, &mut sources),
        Command::ListSources(args) => run_list_sources(args, &sources),
    }
}
