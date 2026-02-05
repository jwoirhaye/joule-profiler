use crate::aggregate::sensor_result::SensorResult;
use crate::orchestrator::error::OrchestratorError;
use crate::source::types::SourceEvent;
use crate::source::{MetricSource, MetricSourceError};
use futures::future::try_join_all;
use tokio::sync::mpsc::error::SendError;
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

    pub async fn reset(&mut self) -> Result<(), OrchestratorError> {
        self.send_event(SourceEvent::Reset).await
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
    ///
    /// If an error is encountered in a source, then the worker is aborted and the error is returned
    async fn send_event(&mut self, event: SourceEvent) -> Result<(), OrchestratorError> {
        let futures: Vec<_> = self
            .senders
            .iter_mut()
            .enumerate()
            .map(|(i, tx)| async move { tx.send(event).await.map_err(|send_err| (i, send_err)) })
            .collect();

        if let Err((failed_index, send_err)) = try_join_all(futures).await {
            Err(self.handle_event_error(failed_index, send_err).await)
        } else {
            Ok(())
        }
    }

    async fn handle_event_error(
        &mut self,
        failed_index: usize,
        err: SendError<SourceEvent>,
    ) -> OrchestratorError {
        if let Some(handle) = self.handles.get_mut(failed_index) {
            match handle.await {
                Ok(Ok((_result, _source))) => err.into(),
                Ok(Err(metric_err)) => metric_err.into(),
                Err(join_err) => join_err.into(),
            }
        } else {
            err.into()
        }
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
