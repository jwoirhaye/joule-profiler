use crate::orchestrator::error::OrchestratorError;
use crate::source::error::MetricSourceError;
use thiserror::Error;

/// Top-level error type for JouleProfiler.
///
/// This enum represents all possible errors that can occur during the
/// execution of the profiler from initialization, orchestration, sources errors, aggregation and etc.
#[derive(Debug, Error)]
pub enum JouleProfilerError {
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

    #[error("Process control failed: {0}")]
    ProcessControlFailed(String),

    #[error(transparent)]
    MetricSourceError(#[from] MetricSourceError),

    #[error(transparent)]
    OrchestratorError(#[from] OrchestratorError),
}
