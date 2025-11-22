use anyhow::Result;

use crate::cli::{Cli, Command};
use crate::rapl::{check_os, check_rapl, discover_domains, rapl_base_path};

mod list_domains;
mod phases;
mod simple;

pub use list_domains::run_list_domains;
pub use phases::run_phases;
pub use simple::run_simple;

pub fn run(cli: Cli) -> Result<()> {
    check_os()?;

    let base = rapl_base_path(cli.rapl_path.as_ref());
    check_rapl(&base)?;

    let domains = discover_domains(&base)?;

    match cli.command {
        Command::Simple(args) => run_simple(args, &domains),
        Command::Phases(args) => run_phases(args, &domains),
        Command::ListDomains(args) => run_list_domains(args, &domains),
    }
}
