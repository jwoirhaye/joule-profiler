use thiserror::Error;
use tokio::{sync::mpsc::error::SendError, task::JoinError};

use crate::core::{
    profiler::error::JouleProfilerError,
    source::{error::MetricSourceError, types::SourceEvent},
};

#[derive(Debug, Error)]
pub enum OrchestratorError {
    #[error("Not enough snapshots to retrieve")]
    NotEnoughSnapshots,

    #[error(transparent)]
    JoinError(#[from] JoinError),

    #[error(transparent)]
    SendError(#[from] SendError<SourceEvent>),

    #[error("Metric source error")]
    SourceError {
        #[source]
        err: MetricSourceError,
    },

    #[error("Source {index} disconnected, cause: {cause}")]
    SourceDisconnected {
        index: usize,
        cause: MetricSourceError,
    },
}

impl From<OrchestratorError> for JouleProfilerError {
    fn from(err: OrchestratorError) -> Self {
        Self::Orchestrator { err }
    }
}
