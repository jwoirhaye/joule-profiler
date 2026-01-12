use anyhow::Result;
use enum_dispatch::enum_dispatch;

use crate::{
    config::OutputFormat,
    source::{metric::Snapshot, rapl::Rapl},
};

pub mod metric;
pub mod rapl;

#[enum_dispatch]
pub trait MetricReader {
    fn measure(&mut self) -> Result<()>;
    fn retrieve(&mut self) -> Result<Vec<Snapshot>>;
    fn print_source(&self, format: OutputFormat) -> Result<()>;
}

#[enum_dispatch(MetricReader)]
pub enum MetricSource {
    Rapl(Rapl),
}
