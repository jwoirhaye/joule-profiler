use std::{
    thread::{self, JoinHandle},
    time::Duration,
};

use anyhow::Result;
use crossbeam_channel::{Receiver, Sender, select};
use enum_dispatch::enum_dispatch;
use log::{debug, error, info, trace, warn};
use serde::Serialize;

use crate::source::powercap::Rapl;

pub mod powercap;

#[derive(Serialize, Clone, Debug)]
pub struct Metric {
    pub name: String,
    pub value: u64,
    pub unit: String,
    pub source: String,
}

#[derive(Debug, Clone, Copy)]
pub enum SourceEvent {
    Measure,
    Phase,
    Start,
    Pause,
    Stop,
}

#[enum_dispatch]
pub trait MetricReader {
    fn measure(&mut self) -> Result<()>;

    fn phase(&mut self) -> Result<()>;

    fn retrieve(&mut self) -> Result<Vec<Vec<Metric>>>;

    fn get_sensors(&self) -> Result<Vec<Sensor>>;

    fn get_polling_interval(&self) -> Option<Duration> {
        None
    }

    fn get_name(&self) -> &'static str;
}

#[enum_dispatch(MetricReader)]
#[derive(Clone, Debug)]
pub enum MetricSource {
    Rapl(Rapl),
}

pub struct SourceManager {
    sources: Vec<MetricSource>,
    senders: Vec<Sender<SourceEvent>>,
    handles: Vec<JoinHandle<Result<Vec<Vec<Metric>>>>>,
}

impl SourceManager {
    pub fn new(sources: Vec<MetricSource>) -> Self {
        Self {
            sources,
            senders: Vec::new(),
            handles: Vec::new(),
        }
    }

    pub fn start_workers(&mut self) {
        let sources = self.sources.clone();
        let mut senders = Vec::new();
        let mut handles = Vec::new();

        for mut source in sources {
            let (tx, rx): (Sender<SourceEvent>, Receiver<SourceEvent>) =
                crossbeam_channel::bounded(4);
            senders.push(tx.clone());

            let handle = thread::spawn(move || {
                let poll_interval = source.get_polling_interval();
                let mut polling_active = poll_interval.is_some();
                let interval = poll_interval.unwrap_or(Duration::from_millis(1));

                info!("Worker started for source {:?}", source.get_name());

                loop {
                    select! {
                        recv(rx) -> msg => match msg {
                            Ok(event) => {
                                trace!("Received event {:?} for source {:?}", event, source.get_name());
                                match event {
                                    SourceEvent::Stop => {
                                        info!("Stopping worker for source {:?}", source.get_name());
                                        return source.retrieve();
                                    },
                                    SourceEvent::Phase => {
                                        debug!("Phase event for source {:?}", source.get_name());
                                        if let Err(e) = source.phase() {
                                            error!("Phase error: {:?}", e);
                                        }
                                    },
                                    SourceEvent::Pause => {
                                        debug!("Pausing polling for source {:?}", source.get_name());
                                        polling_active = false;
                                    },
                                    SourceEvent::Start => {
                                        debug!("Starting polling for source {:?}", source.get_name());
                                        if poll_interval.is_some() {
                                            polling_active = true;
                                        }
                                    },
                                    SourceEvent::Measure => {
                                        if let Err(e) = source.measure() {
                                            error!("Measure error: {:?}", e);
                                        }
                                    },
                                }
                            }
                            Err(_) => {
                                warn!("Channel disconnected for source {:?}", source);
                                return source.retrieve();
                            }
                        },
                        default(interval) => {
                            if polling_active {
                                if let Err(e) = source.measure() {
                                    error!("Polling measure error: {:?}", e);
                                } else {
                                    trace!("Polling measure done for source {:?}", source);
                                }
                            }
                        }
                    }
                }
            });
            handles.push(handle);
        }

        self.handles = handles;
        self.senders = senders;
    }

    pub fn list_sensors(&self) -> Result<Vec<Sensor>> {
        let sensors = self
            .sources
            .iter()
            .flat_map(|source| source.get_sensors())
            .flatten()
            .collect();
        Ok(sensors)
    }

    pub fn send_event(&self, event: SourceEvent) -> Result<()> {
        for sender in &self.senders {
            sender.send(event)?;
        }
        Ok(())
    }

    pub fn start(&self) -> Result<()> {
        self.send_event(SourceEvent::Start)
    }
    pub fn measure(&self) -> Result<()> {
        self.send_event(SourceEvent::Measure)
    }
    pub fn phase(&self) -> Result<()> {
        self.send_event(SourceEvent::Phase)
    }
    pub fn pause(&self) -> Result<()> {
        self.send_event(SourceEvent::Pause)
    }
    pub fn stop(&self) -> Result<()> {
        self.send_event(SourceEvent::Stop)
    }

    pub fn join(&mut self) -> Result<Vec<Vec<Metric>>> {
        info!("Stopping all workers");
        self.stop()?;

        let handles = std::mem::take(&mut self.handles);
        let mut all_phases = Vec::new();

        for handle in handles {
            match handle.join() {
                Ok(Ok(phases)) => all_phases.push(phases),
                Ok(Err(e)) => error!("Worker returned error: {:?}", e),
                Err(_) => error!("Worker panicked"),
            }
        }

        info!("All workers joined. Merging phases");
        let max_phases = all_phases.iter().map(|s| s.len()).max().unwrap_or(0);
        let mut merged = Vec::with_capacity(max_phases);

        for i in 0..max_phases {
            let mut phase_metrics = Vec::new();
            for source_phases in &all_phases {
                if let Some(metrics) = source_phases.get(i) {
                    phase_metrics.extend(metrics.clone());
                }
            }
            merged.push(phase_metrics);
        }

        info!("Merged {} phases", merged.len());
        Ok(merged)
    }
}

#[derive(Serialize)]
pub struct Sensor {
    pub name: String,
    pub unit: String,
    pub source: String,
}
