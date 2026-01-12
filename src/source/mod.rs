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

#[derive(Default)]
pub struct SourceManager {
    sources: Vec<MetricSource>,

}

impl SourceManager {
    pub fn new(sources: Vec<MetricSource>) -> Self {
        Self {
            sources
        }
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