use clap::Parser;

use super::CommonArgs;

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

    /// The common arguments between profiler commands
    #[command(flatten)]
    pub common: CommonArgs,
}
