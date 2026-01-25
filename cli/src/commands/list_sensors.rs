use clap::Parser;

/// Arguments for ListSources subcommand
#[derive(Parser, Debug)]
pub struct ListSensorsArgs {
    /// Output as JSON instead of a formatted table
    #[arg(long = "json")]
    pub json: bool,

    /// Output as CSV (header + rows)
    #[arg(long = "csv")]
    pub csv: bool,
}
