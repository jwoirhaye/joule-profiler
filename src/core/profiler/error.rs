use thiserror::Error;

use crate::core::{
    displayer::error::DisplayerError, orchestrator::error::OrchestratorError,
    source::error::MetricSourceError,
};

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
