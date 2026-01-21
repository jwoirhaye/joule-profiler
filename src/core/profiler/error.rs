use thiserror::Error;

use crate::core::{
    displayer::DisplayerError, orchestrator::error::OrchestratorError,
    source::error::MetricSourceError,
};

/// Top-level error type for JouleProfiler.
///
/// This enum represents all possible errors that can occur during the
/// execution of the profiler, from CLI parsing to command execution,
/// metric collection, and result display.
///
/// # Variants
///
/// - `ParseCliArguments` ([`clap::error::Error`]): Failed to parse command-line arguments.
/// - `InvalidIterations` (`usize`): Provided iterations value is invalid (must be >= 1).
/// - `CommandExecutionFailed` (`String`): Failed to execute the target command.
/// - `CommandNotFound` (`String`): The command was not found in the system.
/// - `CommandKilled` (`i32`): The command was killed by a signal (code provided).
/// - `TokenNotFound` (`String`): Expected token not found in program output.
/// - `InvalidTokenOrder` { start, end } (`String`): End token found before start token.
/// - `MultipleTokens` (`String`): Multiple occurrences of a token were found (expected exactly one).
/// - `OutputFileCreationFailed` (`String`): Could not create the specified output file.
/// - `InvalidPattern` (`String`): Provided regex pattern is invalid.
/// - `StdOutCaptureFail`: Failed to capture standard output from the command.
/// - `IoError` ([`std::io::Error`]): Any I/O error encountered during execution.
/// - `MetricSourceError` ([`MetricSourceError`]): Error from a metrics source.
/// - `DisplayerError` ([`DisplayerError`]): Error while displaying metrics.
/// - `OrchestratorError` (`OrchestratorError`): Error from the orchestrator.
#[derive(Debug, Error)]
pub enum JouleProfilerError {
    #[error("Failed to parse CLI arguments")]
    ParseCliArguments(
        #[from]
        #[source]
        clap::error::Error,
    ),

    #[error("Invalid iterations value: {0}. Must be >= 1")]
    InvalidIterations(usize),

    #[error("Failed to execute command: {0}")]
    CommandExecutionFailed(String),

    #[error("Command not found: {0}")]
    CommandNotFound(String),

    #[error("Command killed by signal: {0}")]
    CommandKilled(i32),

    #[error("Token '{0}' not found in program output")]
    TokenNotFound(String),

    #[error("End token '{end}' found before start token '{start}'")]
    InvalidTokenOrder { start: String, end: String },

    #[error("Multiple occurrences of token '{0}' found (expected exactly one)")]
    MultipleTokens(String),

    #[error("Failed to create output file: {0}")]
    OutputFileCreationFailed(String),

    #[error("Invalid regex pattern: {0}")]
    InvalidPattern(String),

    #[error("Stdout capture failed")]
    StdOutCaptureFail,

    #[error("I/O error")]
    IoError(
        #[from]
        #[source]
        std::io::Error,
    ),

    #[error("Metric source error")]
    MetricSourceError(
        #[from]
        #[source]
        MetricSourceError,
    ),

    #[error("Displayer error")]
    DisplayerError(
        #[from]
        #[source]
        DisplayerError,
    ),

    #[error("Orchestrator error")]
    OrchestratorError(
        #[from]
        #[source]
        OrchestratorError,
    ),
}
