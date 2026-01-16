use clap::{ArgAction, Parser, Subcommand};

/// joule-profiler: measure program energy consumption using Intel RAPL
#[derive(Parser, Debug)]
#[command(name = "joule-profiler")]
#[command(version, about = "Measure program energy consumption using Intel RAPL")]
pub struct Cli {
    /// Verbosity (-v, -vv, -vvv)
    #[arg(short = 'v', long = "verbose", action = ArgAction::Count)]
    pub verbose: u8,

    /// Override the base path used to read Intel RAPL counters.
    ///
    /// By default, the profiler reads from:
    ///   /sys/devices/virtual/powercap/intel-rapl
    ///
    /// If not provided, the profiler uses (by priority):
    ///   1. $JOULE_PROFILER_RAPL_PATH (if set)
    ///   2. /sys/devices/virtual/powercap/intel-rapl
    #[arg(long = "rapl-path")]
    pub rapl_path: Option<String>,

    /// Sockets to measure (e.g. 0 or 0,1)
    #[arg(short = 's', long = "sockets")]
    pub sockets: Option<String>,

    #[command(subcommand)]
    pub command: ProfilerCommand,
}

/// Subcommands of joule-profiler
#[derive(Subcommand, Debug)]
pub enum ProfilerCommand {
    /// Standard measurement mode (single or repeated)
    Simple(SimpleArgs),

    /// Phase-based measurement mode (with start/end tokens)
    Phases(PhasesArgs),

    /// List available RAPL energy domains
    ListSensors(ListArgs),
}

/// Fields common to both Simple and Phases modes
#[derive(Parser, Debug)]
pub struct CommonArgs {
    /// Export results as JSON instead of pretty terminal output
    #[arg(long, conflicts_with = "csv")]
    pub json: bool,

    /// Export results as CSV (semicolon-separated values)
    #[arg(long, conflicts_with = "json")]
    pub csv: bool,

    /// Number of iterations (>=1)
    #[arg(short = 'n', long = "iterations")]
    pub iterations: Option<usize>,

    /// Output file for CSV/JSON (else data<TIMESTAMP>.csv/json)
    #[arg(long = "jouleit-file")]
    pub jouleit_file: Option<String>,

    /// Redirect profiled program stdout to this file
    #[arg(short = 'o', long = "output-file")]
    pub output_file: Option<String>,

    /// Command to execute (everything after `--`)
    #[arg(last = true)]
    pub cmd: Vec<String>,

    #[arg(long = "rapl-polling")]
    pub rapl_polling: Option<f64>,
}

/// Arguments for Simple mode
#[derive(Parser, Debug)]
pub struct SimpleArgs {
    #[command(flatten)]
    pub common: CommonArgs,
}

/// Arguments for Phase-based mode
#[derive(Parser, Debug)]
pub struct PhasesArgs {
    /// Regex pattern to detect phase tokens in program output.
    ///
    /// Matches tokens in stdout; if the pattern has a capture group, the
    /// captured text is used as the token name. Energy phases computed:
    ///   - global (START -> END)
    ///   - START -> first_token
    ///   - token_i -> token_i+1
    ///   - last_token -> END
    #[arg(
        long = "token-pattern",
        default_value = "__[A-Z0-9_]+__",
        value_name = "REGEX"
    )]
    pub token_pattern: String,

    #[command(flatten)]
    pub common: CommonArgs,
}

/// Arguments for ListSources subcommand
#[derive(Parser, Debug)]
pub struct ListArgs {
    /// Output as JSON instead of a formatted table
    #[arg(long = "json")]
    pub json: bool,

    /// Output as CSV (header + rows)
    #[arg(long = "csv")]
    pub csv: bool,
}
