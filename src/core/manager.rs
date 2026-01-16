use std::time::Duration;

use anyhow::Result;
use tokio::{
    select,
    sync::mpsc::{Receiver, Sender, channel},
    task::JoinHandle,
    time::{MissedTickBehavior, interval},
};

use crate::{
    core::source::{MetricReader, SourceEvent, SourceResult},
    sources::MetricSource,
};

pub struct SourceManager {
    sources: Vec<MetricSource>,
    senders: Vec<Sender<SourceEvent>>,
    handles: Vec<JoinHandle<Result<SourceResult>>>,
}

impl SourceManager {
    pub fn new(sources: Vec<MetricSource>) -> Self {
        Self {
            sources,
            senders: Vec::new(),
            handles: Vec::new(),
        }
    }

    /// Start the metrics sources worker threads.
    pub async fn start_workers(&mut self) {
        let sources = self.sources.clone();
        let mut senders = Vec::new();
        let mut handles = Vec::new();

        for source in sources {
            let (tx, rx) = channel(4);
            senders.push(tx.clone());

            let handle = tokio::spawn(async move {
                let poll_interval = source.get_polling_interval();

                match poll_interval {
                    Some(interval) => run_worker_with_polling(source, rx, interval).await,
                    None => run_worker_event_only(source, rx).await,
                }
            });

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

/// Start a worker without polling.
async fn run_worker_event_only<S: MetricReader>(
    mut source: S,
    mut rx: Receiver<SourceEvent>,
) -> Result<SourceResult> {
    loop {
        match rx.recv().await {
            Some(SourceEvent::Stop) => return source.retrieve(),
            Some(event) => handle_event_no_polling(&mut source, event)?,
            _ => {}
        }
    }
}

/// Handle an event for a no-polling worker (only phase and measure events supported).
fn handle_event_no_polling<S: MetricReader>(source: &mut S, event: SourceEvent) -> Result<()> {
    match event {
        SourceEvent::Phase => {
            source.phase()?;
        }
        SourceEvent::Measure => {
            source.measure()?;
        }
        _ => {}
    }
    Ok(())
}

async fn run_worker_with_polling<S: MetricReader>(
    mut source: S,
    mut rx: Receiver<SourceEvent>,
    polling_interval: Duration,
) -> Result<SourceResult> {
    let mut polling_active = true;

    let mut reload_timer = interval(polling_interval);
    reload_timer.set_missed_tick_behavior(MissedTickBehavior::Delay);

    loop {
        select! {
            Some(event) = rx.recv() => {
                match event {
                    SourceEvent::Start => polling_active = true,
                    SourceEvent::Stop => polling_active = false,
                    SourceEvent::Join => return source.retrieve(),
                    SourceEvent::Measure => {
                        source.measure()?;
                    },
                    SourceEvent::Phase => {
                        source.phase()?;
                    },
                }
            }
            _ = reload_timer.tick() => {
                if polling_active {
                    source.measure()?;
                }
            }
        }
    }
}
