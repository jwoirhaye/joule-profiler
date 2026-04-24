use crate::commands::profile::ProfileArgs;
use clap::Subcommand;

pub mod profile;

/// Subcommands of joule-profiler.
#[derive(Subcommand, Debug)]
pub enum ProfilerCommand {
    /// Profiling mode, executes a command and profiles it.
    Profile(ProfileArgs),

    /// List available sensors.
    ListSensors,
}
