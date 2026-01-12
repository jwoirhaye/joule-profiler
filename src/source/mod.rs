use std::{sync::mpsc::{Sender, channel}, thread::JoinHandle};

use anyhow::Result;
use enum_dispatch::enum_dispatch;

use crate::{
    config::OutputFormat,
    source::{metric::{Snapshot}, rapl::Rapl},
};

pub mod metric;
pub mod rapl;

#[enum_dispatch]
pub trait MetricReader {
    fn measure(&mut self) -> Result<()>;
    fn retrieve(&mut self) -> Result<Vec<Snapshot>>;
    fn print_available_sensors(&self, format: OutputFormat) -> Result<()>;
}

#[enum_dispatch(MetricReader)]
pub enum MetricSource {
    Rapl(Rapl),
}

enum Event {
    Stop,
    Measure
}

struct SourceHandle {
    tx: Sender<Event>,
    handle: JoinHandle<Result<Vec<Snapshot>>>
}

#[derive(Default)]
pub struct SourceManager {
    sources: Vec<MetricSource>,
    handles: Vec<SourceHandle>,
}

impl SourceManager {
    pub fn new(sources: Vec<MetricSource>) -> Self {
        Self {
            sources,
            handles: Vec::new()
        }
    }

    pub fn start(&mut self) {
        let sources = std::mem::take(&mut self.sources);
        self.handles = sources.into_iter().map(|mut source| {
            let (tx, rx) = channel();
            let join_handle = std::thread::spawn(move || {
                loop {
                    if let Ok(event) = rx.recv() {
                        match event {
                            Event::Stop => break,
                            Event::Measure => source.measure()?,
                        };
                    }
                }
                source.retrieve()
            });
            SourceHandle { handle: join_handle, tx }
        }).collect()
    }

    pub fn add_source<T: Into<MetricSource>>(&mut self, source: T) {
        self.sources.push(source.into());
    }

    pub fn measure(&mut self) -> Result<()> {
        for source in &mut self.sources {
            source.measure()?;
        }
        Ok(())
    }

    pub fn retrieve(&mut self) -> Result<Vec<Vec<Snapshot>>> {
        let mut snapshots = Vec::with_capacity(self.sources.len());
        for source in &mut self.sources {
            snapshots.push(source.retrieve()?);
        }
        Ok(snapshots)
    }

    pub fn print_available_sensors(&self, format: OutputFormat) -> Result<()> {
        for source in &self.sources {
            source.print_available_sensors(format)?;
        }
        Ok(())
    }
}