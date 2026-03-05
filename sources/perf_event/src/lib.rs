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
use log::{debug, info, trace};
use perf_event::{Builder, Counter, Group, events::Hardware};

use crate::{
    error::PerfEventError,
    event::{EVENTS, Event},
    snapshot::{Phase, Snapshot},
};

mod error;
mod event;
mod snapshot;

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
    /// Begin phase snapshot
    begin_snapshot: Option<Snapshot>,
    /// Last measurement snapshot for delta calculation
    last_snapshot: Option<Snapshot>,
}

impl PerfEvent {
    /// Creates a new uninitialized perf_event source.
    ///
    /// Call `init()` with a PID before measuring.
    pub fn new() -> Result<Self> {
        debug!("Creating new perf_event source");
        Ok(Self {
            group: Group::new()?,
            perf_counters: HashMap::default(),
            begin_snapshot: None,
            last_snapshot: None,
        })
    }

    /// Initialize the perf_event group for a specific process.
    fn init_group(&mut self, pid: i32) -> Result<()> {
        debug!("Initializing perf_event group for PID {}", pid);
        self.group = Group::builder()
            .observe_pid(pid)
            .inherit(true)
            .build_group()
            .map_err(PerfEventError::from)?;
        trace!("perf_event group successfully created");
        Ok(())
    }

    /// Add all configured event counters to the group.
    fn init_counters(&mut self, events: &[Event], pid: i32) -> Result<()> {
        debug!("Adding {} performance counters", events.len());
        for event in events {
            trace!("Adding counter: {:?}", event);
            let counter = self.group.add(
                Builder::new(Hardware::from(*event))
                    .inherit(true)
                    .observe_pid(pid),
            )?;
            self.perf_counters.insert(*event, counter);
        }
        info!("Initialized {} hardware performance counters", events.len());
        Ok(())
    }
}

impl MetricReader for PerfEvent {
    type Type = Phase;

    type Error = PerfEventError;

    /// Read current counter values and compute delta since last measurement.
    async fn measure(&mut self) -> Result<()> {
        trace!("Reading perf_event counters");
        let data = self.group.read()?;

        let new_snapshot = Snapshot {
            metrics: self
                .perf_counters
                .iter()
                .map(|(event, counter)| {
                    let value = data
                        .get(counter)
                        .map(|d| d.value())
                        .ok_or(PerfEventError::ErrorReadingCounter(*event))?;
                    Ok((*event, value))
                })
                .collect::<Result<HashMap<_, _>>>()?,
        };

        if self.begin_snapshot.is_none() {
            self.begin_snapshot = Some(new_snapshot);
        } else {
            self.last_snapshot = Some(new_snapshot);
        }

        Ok(())
    }

    /// Retrieve and consume the last measurement snapshot.
    async fn retrieve(&mut self) -> Result<Self::Type> {
        if let Some(begin) = self.begin_snapshot.take()
            && let Some(end) = self.last_snapshot.take()
        {
            self.begin_snapshot = Some(end.clone());
            Ok(Phase { begin, end })
        } else {
            Err(PerfEventError::NotEnoughSamples)
        }
    }

    /// Returns available hardware performance counter sensors.
    fn get_sensors(&self) -> Result<Sensors> {
        trace!("Building perf_event sensor list");
        let sensors: Sensors = EVENTS
            .iter()
            .map(|event| {
                trace!("Registering sensor: {}", event);
                Sensor {
                    name: event.to_string(),
                    source: Self::get_name().to_string(),
                    unit: event.unit(),
                }
            })
            .collect();

        debug!("Registered {} perf_event sensors", sensors.len());
        Ok(sensors)
    }

    fn get_name() -> &'static str {
        "perf_event"
    }

    /// Initialize counters for a specific process and start monitoring.
    async fn init(&mut self, pid: i32) -> Result<()> {
        info!("Initializing perf_event source for PID {}", pid);
        self.init_group(pid)?;
        self.init_counters(EVENTS, pid)?;
        self.group.enable()?;
        debug!("perf_event counters enabled");
        Ok(())
    }

    /// Reset the current counters.
    async fn reset(&mut self) -> Result<()> {
        self.perf_counters = HashMap::new();
        self.last_snapshot = None;
        self.begin_snapshot = Default::default();
        Ok(())
    }

    /// Convert raw counter values to metrics with metadata.
    fn to_metrics(&self, result: Self::Type) -> Result<Metrics> {
        trace!(
            "Converting {} counters to metrics",
            result.begin.metrics.len()
        );
        let diff = result.diff();
        Ok(diff
            .metrics
            .into_iter()
            .map(|(event, counter)| Metric {
                name: event.to_string(),
                source: Self::get_name().to_string(),
                value: counter,
                unit: event.unit(),
            })
            .collect())
    }
}
