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

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Standard measurement mode (single or repeated)
    Simple(SimpleArgs),

    /// Phase-based measurement mode (with start/end tokens)
    Phases(PhasesArgs),

    /// List available RAPL energy domains
    ListDomains(ListArgs),
}

#[derive(Parser, Debug)]
pub struct ListArgs {
    /// Sockets to filter (optional), e.g. "0" or "0,1"
    #[arg(short = 's', long = "sockets")]
    pub sockets: Option<String>,

    /// Output as JSON instead of a formatted table
    #[arg(long = "json")]
    pub json: bool,

    /// Output as CSV (header + rows)
    #[arg(long = "csv")]
    pub csv: bool,
}

#[derive(Parser, Debug)]
pub struct SimpleArgs {
    /// Export results as JSON instead of pretty terminal output
    #[arg(long = "json")]
    pub json: bool,

    /// Export results as CSV (semicolon-separated values)
    ///
    /// For single measurements: outputs to stdout
    /// For iterations: outputs to file (--jouleit-file or data<TIMESTAMP>.csv)
    #[arg(long = "csv")]
    pub csv: bool,

    /// Number of iterations (>=1).
    ///
    /// When provided, the command is executed N times and all
    /// measurements are exported to a file:
    ///   - CSV if --csv is set
    ///   - JSON if --json is set
    ///   - Terminal output otherwise
    #[arg(short = 'n', long = "iterations")]
    pub iterations: Option<usize>,

    /// Output file for CSV/JSON (else data<TIMESTAMP>.csv/json)
    #[arg(long = "jouleit-file")]
    pub jouleit_file: Option<String>,

    /// Sockets to measure (e.g. 0 or 0,1)
    #[arg(short = 's', long = "sockets")]
    pub sockets: Option<String>,

    /// Redirect profiled program stdout to this file
    #[arg(short = 'o', long = "output-file")]
    pub output_file: Option<String>,

    /// Command to execute (everything after `--`)
    #[arg(last = true)]
    pub cmd: Vec<String>,
}

#[derive(Parser, Debug)]
pub struct PhasesArgs {
    /// Start token printed by the program on stdout
    #[arg(long = "token-start", default_value = "__WORK_START__")]
    pub token_start: String,

    /// End token printed by the program on stdout
    #[arg(long = "token-end", default_value = "__WORK_END__")]
    pub token_end: String,

    /// Export results as JSON (default: terminal pretty print)
    #[arg(long = "json")]
    pub json: bool,

    /// Export results as CSV (semicolon-separated values)
    ///
    /// For single measurements: outputs to file (--jouleit-file or data<TIMESTAMP>.csv)
    /// For iterations: outputs to file with iteration and phase_name columns
    #[arg(long = "csv")]
    pub csv: bool,

    /// Number of iterations (>=1).
    ///
    /// When provided, the command is executed N times and
    /// each iteration is measured using phases (global, pre, work, post).
    #[arg(short = 'n', long = "iterations")]
    pub iterations: Option<usize>,

    /// Output file for CSV/JSON (else data<TIMESTAMP>.csv/json)
    #[arg(long = "jouleit-file")]
    pub jouleit_file: Option<String>,

    #[arg(short = 's', long = "sockets")]
    pub sockets: Option<String>,

    /// Redirect profiled program stdout to this file
    #[arg(short = 'o', long = "output-file")]
    pub output_file: Option<String>,

    /// Command to execute (everything after `--`)
    #[arg(last = true)]
    pub cmd: Vec<String>,
}
