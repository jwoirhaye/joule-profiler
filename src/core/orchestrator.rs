use anyhow::Result;
use tokio::{
    sync::mpsc::{Sender, channel},
    task::JoinHandle,
};

use crate::{
    core::source::{MetricSourceWorker, SensorResult, SourceEvent},
    error::JouleProfilerError,
};

pub struct SourceOrchestrator {
    senders: Vec<Sender<SourceEvent>>,
    handles: Vec<JoinHandle<Result<SensorResult>>>,
}

impl SourceOrchestrator {
    pub fn new() -> Self {
        Self {
            senders: Vec::new(),
            handles: Vec::new(),
        }
    }

    /// Start the metrics sources worker threads.
    #[inline]
    pub async fn start(&mut self, sources: Vec<Box<dyn MetricSourceWorker>>) {
        let nb_sources = sources.len();
        let mut senders = Vec::with_capacity(nb_sources);
        let mut handles = Vec::with_capacity(nb_sources);

        for source in sources {
            let (tx, rx) = channel(4);
            let handle = tokio::spawn(async move { source.run(rx).await });

            senders.push(tx.clone());
            handles.push(handle);
        }

        self.handles = handles;
        self.senders = senders;
    }

    /// Start the polling of a metrics source if enabled.
    #[inline]
    pub async fn start_polling(&self) -> Result<()> {
        self.send_event(SourceEvent::StartPolling).await
    }

    /// Measure the metrics of each metrics source.
    #[inline]
    pub async fn measure(&self) -> Result<()> {
        self.send_event(SourceEvent::Measure).await
    }

    /// Initialize a new phase for each metrics source.
    #[inline]
    pub async fn new_phase(&self) -> Result<()> {
        self.send_event(SourceEvent::NewPhase).await
    }

    /// Pause the polling of a metrics source if enabled.
    #[inline]
    pub async fn stop_polling(&self) -> Result<()> {
        self.send_event(SourceEvent::StopPolling).await
    }

    /// Initialize a new iteration for each metrics source.
    #[inline]
    pub async fn new_iteration(&self) -> Result<()> {
        self.send_event(SourceEvent::NewIteration).await
    }

    /// Gracefully shutdown all the workers.
    pub async fn retrieve(&mut self) -> Result<SensorResult> {
        self.join().await?;

        let handles = std::mem::take(&mut self.handles);
        let mut sources_results = Vec::new();

        for handle in handles {
            handle
                .await?
                .map(|source_result| sources_results.push(source_result))?;
        }

        let merged_results =
            SensorResult::merge(sources_results).ok_or(JouleProfilerError::NotEnoughSnapshots)?;
        let iterations = merged_results.iterations;
        let result = SensorResult::new(iterations);
        Ok(result)
    }

    /// Stop the worker thread of each metrics sources to join threads gracefully.
    #[inline]
    async fn join(&self) -> Result<()> {
        self.send_event(SourceEvent::JoinWorker).await
    }

    /// Send an event to each metrics source.
    async fn send_event(&self, event: SourceEvent) -> Result<()> {
        for sender in &self.senders {
            sender.send(event).await?;
        }
        Ok(())
    }
}
