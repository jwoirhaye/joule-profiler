use anyhow::Result;

use crate::{
    cli::{Cli, Command},
    source::{
        SourceManager, rapl::{Rapl, get_domains}
    },
};

mod list_domains;
mod phases;
mod simple;

pub use list_domains::run_list_sensors;
pub use phases::run_phases;
pub use simple::run_simple;

pub fn run(cli: Cli) -> Result<()> {
    let domains = get_domains(cli.rapl_path.as_deref(), cli.sockets.as_deref())?;

    let mut source_manager = SourceManager::default();

    let rapl = Rapl::new(domains)?;
    source_manager.add_source(rapl);


    match cli.command {
        Command::Simple(args) => run_simple(args, &mut source_manager),
        Command::Phases(args) => run_phases(args, &mut source_manager),
        Command::ListSources(args) => run_list_sensors(args, &source_manager),
    }
}
