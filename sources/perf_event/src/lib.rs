//! `perf_event` source for hardware performance counters.
//!
//! Measures CPU cycles, instructions, cache misses, and branch misses
//! using Linux `perf_event` subsystem.
//!
//! Note: Counters are created individually (not grouped) because
//! `inherit(true)` is incompatible with `perf_event` groups on Linux.

use std::collections::HashMap;

use joule_profiler_core::{
    sensor::{Sensor, Sensors},
    source::MetricReader,
    types::{Metric, Metrics},
    unit::{MetricUnit, Unit, UnitPrefix},
};
use log::{debug, info, trace};
use perf_event::{Builder, Counter, events::Hardware};

use crate::{
    error::PerfEventError,
    event::{EVENTS, Event},
    snapshot::{Phase, Snapshot},
};

mod error;
mod event;
mod snapshot;

type Result<T> = std::result::Result<T, PerfEventError>;

const PERF_EVENT_METRIC_UNIT: MetricUnit = MetricUnit {
    prefix: UnitPrefix::None,
    unit: Unit::Count,
};

/// Hardware performance counter source using `perf_event`.
///
/// Tracks CPU performance metrics (cycles, instructions, cache/branch misses)
/// for a specific process.
pub struct PerfEvent {
    /// Active performance counters by event type
    perf_counters: HashMap<Event, Counter>,

    begin_snapshot: Option<Snapshot>,

    last_snapshot: Option<Snapshot>,
}

impl PerfEvent {
    /// Creates a new uninitialized `perf_event` source.
    ///
    /// Call `init()` with a PID before measuring.
    pub fn new() -> Result<Self> {
        debug!("Creating new perf_event source");
        Ok(Self {
            perf_counters: HashMap::default(),
            begin_snapshot: None,
            last_snapshot: None,
        })
    }

    /// Create individual counters for all configured events.
    ///
    /// Each counter is built separately with `inherit(true)` and `observe_pid`,
    /// since grouped counters do not support inheritance.
    fn init_counters(&mut self, events: &[Event], pid: i32) -> Result<()> {
        debug!("Adding {} individual performance counters", events.len());
        for event in events {
            trace!("Building counter: {event:?}");
            let counter = Builder::new(Hardware::from(*event))
                .inherit(true)
                .observe_pid(pid)
                .build()?;
            self.perf_counters.insert(*event, counter);
        }
        info!("Initialized {} hardware performance counters", events.len());
        Ok(())
    }

    /// Enable all counters.
    fn enable_all(&mut self) -> Result<()> {
        for (event, counter) in &mut self.perf_counters {
            trace!("Enabling counter: {event:?}");
            counter.enable()?;
        }
        debug!("All perf_event counters enabled");
        Ok(())
    }
}

impl MetricReader for PerfEvent {
    type Type = Phase;

    type Error = PerfEventError;

    /// Read current counter values and compute delta since last measurement.
    async fn measure(&mut self) -> Result<()> {
        trace!("Reading perf_event counters");

        let new_snapshot = Snapshot {
            metrics: self
                .perf_counters
                .iter_mut()
                .map(|(event, counter)| {
                    let value = counter
                        .read()
                        .map_err(|_| PerfEventError::ErrorReadingCounter(*event))?;
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
                trace!("Registering sensor: {event}");
                Sensor {
                    name: event.to_string(),
                    source: Self::get_name().to_string(),
                    unit: PERF_EVENT_METRIC_UNIT,
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
        info!("Initializing perf_event source for PID {pid}");
        self.init_counters(EVENTS, pid)?;
        self.enable_all()?;
        Ok(())
    }

    /// Reset the current counters.
    async fn reset(&mut self) -> Result<()> {
        self.perf_counters = HashMap::new();
        self.last_snapshot = None;
        self.begin_snapshot = Option::default();
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
                unit: PERF_EVENT_METRIC_UNIT,
            })
            .collect())
    }
}
