use thiserror::Error;
use tokio::{sync::mpsc::error::SendError, task::JoinError};

use crate::core::source::{error::MetricSourceError, types::SourceEvent};

#[derive(Debug, Error)]
pub enum OrchestratorError {
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

    #[error("Metric source error")]
    SourceError(
        #[from]
        #[source]
        MetricSourceError,
    ),

    #[error("Source {index} disconnected, cause: {cause}")]
    SourceDisconnected {
        index: usize,
        #[source]
        cause: MetricSourceError,
    },
}
