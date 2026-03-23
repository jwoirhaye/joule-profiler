use crate::source::MetricSourceError;
use crate::source::types::SourceEvent;
use thiserror::Error;
use tokio::{sync::mpsc::error::SendError, task::JoinError};

/// Errors that can occur during orchestration.
#[derive(Debug, Error)]
pub enum OrchestratorError {
    /// Returned when the snapshot buffer contains fewer entries than required to create results.
    #[error("Not enough snapshots to retrieve")]
    NotEnoughSnapshots,

    #[error("Join error")]
    JoinError(
        #[from]
        #[source]
        JoinError,
    ),

    #[error("Send error")]
    SendError(
        #[from]
        #[source]
        SendError<SourceEvent>,
    ),

    #[error(transparent)]
    MetricSourceError(#[from] MetricSourceError),
}
