use anyhow::Result;
use tokio::{
    sync::mpsc::{Sender, channel},
    task::JoinHandle,
};

use crate::{
    core::source::{MetricReader, MetricSource, SourceEvent, SourceResult},
    sources::MetricSourceType,
};

pub struct SourceManager {
    sources: Vec<MetricSourceType>,
    senders: Vec<Sender<SourceEvent>>,
    handles: Vec<JoinHandle<Result<SourceResult>>>,
}

impl SourceManager {
    pub fn new(sources: Vec<MetricSourceType>) -> Self {
        Self {
            sources,
            senders: Vec::new(),
            handles: Vec::new(),
        }
    }

    /// Start the metrics sources worker threads.
    pub async fn start_workers(&mut self) {
        // let sources = self
        //     .sources
        //     .iter()
        //     .cloned()
        //     .map(|source| MetricSource::new(source.into()));

        // MetricSource::new(self.sources[0].into());

        let mut senders = Vec::new();
        let mut handles = Vec::new();

        for mut source in self.sources {
            let (tx, rx) = channel(4);
            senders.push(tx.clone());

            let handle = tokio::spawn(async move { MetricSourceType::into(source).run_worker(rx).await });

            handles.push(handle);
        }

        self.handles = handles;
        self.senders = senders;
    }

    /// Start the polling of a metrics source if enabled.
    pub async fn start(&self) -> Result<()> {
        self.send_event(SourceEvent::Start).await
    }

    /// Measure the metrics of each metrics source.
    pub async fn measure(&self) -> Result<()> {
        self.send_event(SourceEvent::Measure).await
    }

    /// Initialize a new phase for each metrics source.
    pub async fn phase(&self) -> Result<()> {
        self.send_event(SourceEvent::Phase).await
    }

    /// Pause the polling of a metrics source if enabled.
    pub async fn stop(&self) -> Result<()> {
        self.send_event(SourceEvent::Stop).await
    }

    /// Stop the worker thread of each metrics sources to join threads gracefully.
    pub async fn join(&self) -> Result<()> {
        self.send_event(SourceEvent::Join).await
    }

    /// Gracefully shutdown all the workers.
    pub async fn retrieve(&mut self) -> Result<SourceResult> {
        self.join().await?;

        let handles = std::mem::take(&mut self.handles);
        let mut all_phases = Vec::new();

        for handle in handles {
            handle
                .await?
                .map(|source_result| all_phases.push(source_result))?;
        }

        let max_phases = all_phases
            .iter()
            .map(|source_result| source_result.measures.len())
            .max()
            .unwrap_or(0);
        let mut merged = Vec::with_capacity(max_phases);

        let mut measure_count = 0;
        let mut measure_delta = 0;

        for i in 0..max_phases {
            let mut phase_metrics = Vec::new();
            for source_result in &all_phases {
                measure_count += source_result.count;
                measure_delta += source_result.measure_delta;
                if let Some(measures) = source_result.measures.get(i) {
                    phase_metrics.extend(measures.clone());
                }
            }
            merged.push(phase_metrics);
        }

        let nb_sources = all_phases.len();
        measure_count /= nb_sources as u64;
        measure_delta /= nb_sources as u128;

        Ok(SourceResult {
            measures: merged,
            count: measure_count,
            measure_delta,
        })
    }

    /// Send an event to each metrics source.
    async fn send_event(&self, event: SourceEvent) -> Result<()> {
        for sender in &self.senders {
            sender.send(event).await?;
        }
        Ok(())
    }
}
