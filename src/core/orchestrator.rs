use anyhow::Result;
use tokio::{
    sync::mpsc::{Sender, channel},
    task::JoinHandle,
};

use crate::{
    core::{
        sensor::Sensors,
        source::{
            GetSensorsTrait, MetricReader, MetricSource, MetricSourceWorker, SensorResult,
            SourceEvent,
        },
    },
    error::JouleProfilerError,
};

pub struct SourceOrchestrator {
    sources: Vec<Box<dyn MetricSourceWorker>>,
    senders: Vec<Sender<SourceEvent>>,
    handles: Vec<JoinHandle<Result<SensorResult>>>,
}

impl SourceOrchestrator {
    pub fn new() -> Self {
        Self {
            sources: Vec::new(),
            senders: Vec::new(),
            handles: Vec::new(),
        }
    }

    /// Start the metrics sources worker threads.
    pub async fn start(&mut self) {
        let mut senders = Vec::new();
        let mut handles = Vec::new();

        let sources = std::mem::take(&mut self.sources);

        for source in sources {
            let (tx, rx) = channel(4);
            let handle = tokio::spawn(async move { source.run(rx).await });

            senders.push(tx.clone());
            handles.push(handle);
        }

        self.handles = handles;
        self.senders = senders;
    }

    pub fn list_sensors(&self) -> Result<Sensors> {
        let sensors = self
            .sources
            .iter()
            .map(|source| source.list_sensors())
            .collect::<Result<Vec<Sensors>>>()?
            .into_iter()
            .flatten()
            .collect();
        Ok(sensors)
    }

    pub fn add_source<T>(&mut self, reader: T)
    where
        T: MetricReader + GetSensorsTrait + Send + 'static,
        T::Type: Send,
    {
        let source = MetricSource::new(reader);
        self.sources.push(Box::new(source));
    }

    /// Start the polling of a metrics source if enabled.
    pub async fn start_polling(&self) -> Result<()> {
        self.send_event(SourceEvent::StartPolling).await
    }

    /// Measure the metrics of each metrics source.
    pub async fn measure(&self) -> Result<()> {
        self.send_event(SourceEvent::Measure).await
    }

    /// Initialize a new phase for each metrics source.
    pub async fn new_phase(&self) -> Result<()> {
        self.send_event(SourceEvent::NewPhase).await
    }

    /// Pause the polling of a metrics source if enabled.
    pub async fn stop_polling(&self) -> Result<()> {
        self.send_event(SourceEvent::StopPolling).await
    }

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

        let nb_sources = sources_results.len();

        let merged_results =
            SensorResult::merge(sources_results).ok_or(JouleProfilerError::NotEnoughSnapshots)?;
        let measure_count = merged_results.count / nb_sources as u64;
        let measure_delta = merged_results.measure_delta / nb_sources as u64;
        let iterations = merged_results.iterations;

        Ok(SensorResult {
            iterations,
            count: measure_count,
            measure_delta,
        })
    }

    /// Stop the worker thread of each metrics sources to join threads gracefully.
    async fn join(&self) -> Result<()> {
        self.send_event(SourceEvent::Join).await
    }

    /// Send an event to each metrics source.
    async fn send_event(&self, event: SourceEvent) -> Result<()> {
        for sender in &self.senders {
            sender.send(event).await?;
        }
        Ok(())
    }
}
