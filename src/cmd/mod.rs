use anyhow::Result;

use crate::{cli::{Cli, Command}, source::{MetricSource, rapl::{Rapl, get_domains}}};

mod list_domains;
mod phases;
mod simple;

pub use list_domains::run_list_domains;
pub use phases::run_phases;
pub use simple::run_simple;

pub fn run(cli: Cli) -> Result<()> {
    let domains = get_domains(cli.rapl_path.as_deref(), cli.sockets.as_deref())?;
    
    let rapl = Rapl::new(domains)?;
    let sources = vec![rapl.into()];

    match cli.command {
        Command::Simple(args) => run_simple(args, &sources),
        Command::Phases(args) => run_phases(args, &sources),
        // Command::ListDomains(args) => run_list_domains(args, rapl.domains),
        _ => Ok(())
    }
}
