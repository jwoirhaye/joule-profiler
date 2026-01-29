use crate::aggregate::sensor_result::SensorResult;
use crate::orchestrator::error::OrchestratorError;
use crate::source::types::SourceEvent;
use crate::source::{MetricSource, MetricSourceError};
use log::error;
use tokio::{sync::mpsc::Sender, task::JoinHandle};

pub mod error;

/// The handle describing the return type of a source worker
type Handle = JoinHandle<Result<(SensorResult, Box<dyn MetricSource>), MetricSourceError>>;

/// Orchestrates metrics sources and manages their worker threads
#[derive(Default)]
pub struct SourceOrchestrator {
    /// The event channels sender
    senders: Vec<Sender<SourceEvent>>,

    /// The handles of the worker tasks
    handles: Vec<Handle>,
}

impl SourceOrchestrator {
    /// Creates a new source orchestrator
    pub fn new() -> Self {
        Default::default()
    }

    /// Start the metrics sources worker threads
    #[inline]
    pub async fn run(&mut self, sources: Vec<Box<dyn MetricSource>>) {
        let nb_sources = sources.len();
        let mut senders = Vec::with_capacity(nb_sources);
        let mut handles = Vec::with_capacity(nb_sources);

        for source in sources {
            let (handle, tx) = source.run();
            senders.push(tx);
            handles.push(handle);
        }

        self.handles = handles;
        self.senders = senders;
    }

    /// Measure the metrics of each metrics source
    #[inline]
    pub async fn measure(&mut self) -> Result<(), OrchestratorError> {
        self.send_event(SourceEvent::Measure).await
    }

    /// Initialize a new phase for each metrics source
    #[inline]
    pub async fn new_phase(&mut self) -> Result<(), OrchestratorError> {
        self.send_event(SourceEvent::NewPhase).await
    }

    /// Initialize a new iteration for each metrics source
    #[inline]
    pub async fn new_iteration(&mut self) -> Result<(), OrchestratorError> {
        self.send_event(SourceEvent::NewIteration).await
    }

    #[inline]
    pub async fn start(&mut self) -> Result<(), OrchestratorError> {
        self.send_event(SourceEvent::Start).await
    }

    #[inline]
    pub async fn stop(&mut self) -> Result<(), OrchestratorError> {
        self.send_event(SourceEvent::Stop).await
    }

    /// Retrieve and merge results from all sources
    pub async fn finalize(
        &mut self,
    ) -> Result<(SensorResult, Vec<Box<dyn MetricSource>>), OrchestratorError> {
        let (results, sources) = self.join_all().await?;
        let merged = SensorResult::merge(results).ok_or(OrchestratorError::NotEnoughSnapshots)?;
        Ok((merged, sources))
    }

    /// Stop the worker thread of each metrics sources to join threads gracefully.
    #[inline]
    async fn join(&mut self) -> Result<(), OrchestratorError> {
        self.send_event(SourceEvent::JoinWorker).await
    }

    /// Send an event to all metrics sources
    async fn send_event(&mut self, event: SourceEvent) -> Result<(), OrchestratorError> {
        for i in 0..self.senders.len() {
            if let Err(sender_error) = self.senders[i].send(event).await {
                error!("Receiver {} disconnected", i);

                let _ = self.senders.swap_remove(i);
                let handle = self.handles.swap_remove(i);

                let err = match handle.await {
                    Ok(Err(err)) => err.into(),
                    Err(err) => OrchestratorError::JoinError(err),
                    _ => OrchestratorError::SendError(sender_error),
                };

                return Err(err);
            }
        }

        Ok(())
    }

    /// Join all workers and collect results
    async fn join_all(
        &mut self,
    ) -> Result<(Vec<SensorResult>, Vec<Box<dyn MetricSource>>), OrchestratorError> {
        self.join().await?;

        let handles = std::mem::take(&mut self.handles);

        let mut results = Vec::with_capacity(handles.len());
        let mut sources = Vec::with_capacity(handles.len());

        for handle in handles {
            let (result, source) = handle.await??;
            results.push(result);
            sources.push(source);
        }

        Ok((results, sources))
    }
}
