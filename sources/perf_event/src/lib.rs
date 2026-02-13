//! perf_event source for hardware performance counters.
//!
//! Measures CPU cycles, instructions, cache misses, and branch misses
//! using Linux perf_event subsystem.

use std::collections::HashMap;

use joule_profiler_core::{
    sensor::{Sensor, Sensors},
    source::MetricReader,
    types::{Metric, Metrics},
};
use perf_event::{Builder, Counter, Group, events::Hardware};

use crate::{
    error::PerfEventError,
    event::{EVENTS, Event},
};

mod error;
mod event;

type Result<T> = std::result::Result<T, PerfEventError>;

/// Hardware performance counter source using perf_event.
///
/// Tracks CPU performance metrics (cycles, instructions, cache/branch misses)
/// for a specific process. Counters are scoped per-process when initialized.
pub struct PerfEvent {
    /// perf_event group managing all counters together
    group: Group,
    /// Active performance counters by event type
    perf_counters: HashMap<Event, Counter>,
    /// Latest counter values snapshot
    snapshot: HashMap<Event, u64>,
}

impl PerfEvent {
    /// Creates a new uninitialized perf_event source.
    ///
    /// Call `init()` with a PID before measuring.
    pub fn new() -> Result<Self> {
        Ok(Self {
            group: Group::new()?,
            perf_counters: HashMap::default(),
            snapshot: Default::default(),
        })
    }

    /// Initialize the perf_event group for a specific process.
    fn init_group(&mut self, pid: i32) -> Result<()> {
        self.group = Group::builder()
            .observe_pid(pid)
            .build_group()
            .map_err(PerfEventError::from)?;
        Ok(())
    }

    /// Add all configured event counters to the group.
    fn init_counters(&mut self, events: &[Event], pid: i32) -> Result<()> {
        for event in events {
            let counter = self
                .group
                .add(Builder::new(Hardware::from(*event)).observe_pid(pid))?;
            self.perf_counters.insert(*event, counter);
        }
        Ok(())
    }
}

impl MetricReader for PerfEvent {
    type Type = HashMap<Event, u64>;

    type Error = PerfEventError;

    /// Read current counter values and reset them.
    ///
    /// Stores the snapshot internally for later retrieval.
    async fn measure(&mut self) -> Result<()> {
        let data = self.group.read()?;
        self.group.reset()?;

        for (event, counter) in &self.perf_counters {
            let counter_value = if let Some(value) = data.get(counter) {
                value.value()
            } else {
                return Err(PerfEventError::ErrorReadingCounter(*event));
            };
            self.snapshot.entry(*event).insert_entry(counter_value);
        }
        Ok(())
    }

    /// Retrieve and consume the last measurement snapshot.
    async fn retrieve(&mut self) -> Result<Self::Type> {
        Ok(self.snapshot.drain().collect())
    }

    /// Returns available hardware performance counter sensors.
    fn get_sensors(&self) -> Result<Sensors> {
        Ok(EVENTS
            .iter()
            .map(|event| Sensor {
                name: event.to_string(),
                source: Self::get_name().to_string(),
                unit: event.unit(),
            })
            .collect())
    }

    fn get_name() -> &'static str {
        "perf_event"
    }

    /// Initialize counters for a specific process and start monitoring.
    async fn init(&mut self, pid: i32) -> Result<()> {
        self.perf_counters = HashMap::new();
        self.init_group(pid)?;
        self.init_counters(EVENTS, pid)?;
        self.group.enable()?;
        Ok(())
    }

    /// Reset all performance counters to zero.
    async fn reset(&mut self) -> Result<()> {
        self.group.reset()?;
        Ok(())
    }

    /// Convert raw counter values to metrics with metadata.
    fn to_metrics(&self, result: Self::Type) -> Metrics {
        result
            .into_iter()
            .map(|(event, counter)| Metric {
                name: event.to_string(),
                source: Self::get_name().to_string(),
                value: counter,
                unit: event.unit(),
            })
            .collect()
    }
}
