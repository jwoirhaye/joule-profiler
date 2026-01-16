use std::time::Duration;

use anyhow::Result;
use enum_dispatch::enum_dispatch;

use crate::core::{metric::Metrics, sensor::Sensor};

#[derive(Debug, Clone, Copy)]
pub enum SourceEvent {
    Measure,
    Phase,
    Start,
    Stop,
    Join,
}

pub struct SourceResult {
    pub measures: Vec<Metrics>,
    pub count: u64,
    pub measure_delta: u128,
}

#[enum_dispatch]
pub trait MetricReader {
    /// Measure the sensors metrics.
    fn measure(&mut self) -> Result<()>;

    /// Initialize a new measure phase.
    fn phase(&mut self) -> Result<()>;

    /// Retrieve all sensors measures.
    fn retrieve(&mut self) -> Result<SourceResult>;

    /// Get all the metric source sensors.
    fn get_sensors(&self) -> Result<Vec<Sensor>>;

    /// Get the polling interval of the metric source if supported.
    fn get_polling_interval(&self) -> Option<Duration> {
        None
    }
}
