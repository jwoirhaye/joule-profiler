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

    #[error("stdout capture failed")]
    StdOutCaptureFail,


    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error("metric source error")]
    Source {
        #[source]
        err: MetricSourceError,
    },

    #[error("displayer error")]
    Displayer {
        #[source]
        err: DisplayerError,
    },

    #[error("orchestrator error")]
    Orchestrator {
        #[source]
        err: OrchestratorError,
    },
}

impl JouleProfilerError {
    pub fn command_not_found(cmd: impl AsRef<str>) -> Self {
        Self::CommandNotFound(cmd.as_ref().to_string())
    }

    pub fn token_not_found(token: impl AsRef<str>) -> Self {
        Self::TokenNotFound(token.as_ref().to_string())
    }
}
