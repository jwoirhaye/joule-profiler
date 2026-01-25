use crate::commands::list_sensors::ListSensorsArgs;
use crate::commands::phases::PhasesArgs;
use clap::Subcommand;

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
