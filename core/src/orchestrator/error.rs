use crate::source::MetricSourceError;
use crate::source::types::SourceEvent;
use thiserror::Error;
use tokio::{sync::mpsc::error::SendError, task::JoinError};

/// Errors that can occur during orchestration.
#[derive(Debug, Error)]
pub enum OrchestratorError {
    /// Returned when the snapshot buffer contains fewer entries than required to create results.
    #[error("Not enough snapshots to retrieve.")]
    NotEnoughSnapshots,

    #[error("No metric sources configured.")]
    NoSourceConfigured,

    /// Happens when an error occur while joining sources.
    #[error("Join error")]
    JoinError(
        #[from]
        #[source]
        JoinError,
    ),

    /// Returned when an error occur when sending an event to the sources channel.
    #[error("Send error")]
    SendError(
        #[from]
        #[source]
        SendError<SourceEvent>,
    ),

    /// An error thrown by a metric source.
    #[error(transparent)]
    MetricSourceError(#[from] MetricSourceError),
}
