use tokio::sync::mpsc::Sender;

use crate::source::{MetricSourceError, types::SourceEvent};

/// It is a wrapper of an event sender to abstract eventing from source
/// and allow to send measure events and not other events like NewPhase.
pub struct SourceEventEmitter {
    tx: Sender<SourceEvent>,
}

impl SourceEventEmitter {
    /// Initialize an event emitter
    pub(crate) fn new(tx: Sender<SourceEvent>) -> Self {
        Self { tx }
    }

    /// Emit a measure event
    pub async fn emit(&mut self) -> Result<(), MetricSourceError> {
        self.tx
            .send(SourceEvent::Measure)
            .await
            .map_err(|_| MetricSourceError::ErrorRetrievingCounters)
    }
}
