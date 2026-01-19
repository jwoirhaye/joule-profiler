use thiserror::Error;
use tokio::{sync::mpsc::error::SendError, task::JoinError};

use crate::core::{profiler::error::JouleProfilerError, source::{SourceEvent, error::MetricSourceError}};

#[derive(Debug, Error)]
pub enum OrchestratorError {
    #[error("Not enough snapshots to retrieve")]
    NotEnoughSnapshots,

    #[error(transparent)]
    JoinError(#[from] JoinError),

    #[error(transparent)]
    SendError(#[from] SendError<SourceEvent>),
    
    #[error("metric source error")]
    Source {
        #[source]
        err: MetricSourceError,
    },
}

impl From<OrchestratorError> for JouleProfilerError {
    fn from(err: OrchestratorError) -> Self {
        Self::Orchestrator { err }
    }
}