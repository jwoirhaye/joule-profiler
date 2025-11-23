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
    /// Regex pattern to detect phase tokens in program output.
    ///
    /// The pattern matches tokens in stdout, and energy is measured between
    /// consecutive matched tokens. If the pattern has a capture group, the
    /// captured text is used as the token name; otherwise the full match is used.
    ///
    /// Examples:
    ///   - "_.*"              : matches _a, _b, _c, etc.
    ///   - "__([A-Z_]+)__"    : matches __INIT__, __WORK__, __CLEANUP__, etc.
    ///   - "\[(\d{2}:\d{2})\]": matches [12:34], [56:78], etc.
    ///
    /// Energy phases computed:
    ///   - global (START -> END)
    ///   - START -> first_token
    ///   - token_i -> token_i+1 (for all consecutive tokens)
    ///   - last_token -> END
    #[arg(
        long = "token-pattern",
        default_value = "__[A-Z0-9_]+__",
        value_name = "REGEX"
    )]
    pub token_pattern: String,

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
    /// each iteration is measured using phases.
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
