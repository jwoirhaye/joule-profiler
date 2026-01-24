use clap::Subcommand;

use crate::cli::commands::{list_sensors::ListSensorsArgs, phases::PhasesArgs};

pub mod list_sensors;
pub mod phases;

/// Subcommands of joule-profiler
#[derive(Subcommand, Debug)]
pub enum ProfilerCommand {
    /// Phase-based measurement mode (with start/end tokens).
    Phases(PhasesArgs),

    /// List available sensors.
    ListSensors(ListSensorsArgs),
}
