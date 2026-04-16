use crate::{
    Result,
    error::PerfEventError,
    event::{EVENTS, Event},
    snapshot::Snapshot,
};
use joule_profiler_core::types::ProcessInfo;
use log::{debug, info, trace};
use perf_event::{Builder, Counter, events::Hardware};
use std::collections::HashMap;

#[cfg_attr(test, mockall::automock)]
pub trait PerfEventHardware: Send {
    fn init_counters(&mut self, process_info: &ProcessInfo) -> Result<()>;
    fn read_snapshot(&mut self) -> Result<Snapshot>;
}

#[derive(Default)]
pub struct PerfEventCounters {
    counters: HashMap<Event, Counter>,
    global_counters: HashMap<u16, Counter>,
}

impl PerfEventHardware for PerfEventCounters {
    /// Create individual counters for all configured events.
    ///
    /// Each counter is built separately with `inherit(true)` and `observe_pid`,
    /// since grouped counters do not support inheritance.
    fn init_counters(&mut self, process_info: &ProcessInfo) -> Result<()> {
        debug!("Adding {} individual performance counters", EVENTS.len());
        for event in EVENTS {
            trace!("Building counter: {event:?}");
            let counter = Builder::new(Hardware::from(*event))
                .inherit(true)
                .observe_pid(process_info.pid)
                .include_hv()
                .enabled(true)
                .include_kernel()
                .build()?;
            self.counters.insert(*event, counter);
        }

        for cpu in &process_info.sched_affinity {
            let counter = Builder::new(Hardware::from(Hardware::CPU_CYCLES))
                .inherit(true)
                .one_cpu(*cpu as usize)
                .any_pid()
                .include_hv()
                .include_kernel()
                .enabled(true)
                .build()?;
            self.global_counters.insert(*cpu, counter);
        }

        info!("Initialized {} hardware performance counters", EVENTS.len());
        Ok(())
    }

    /// Reads all performance counters and returns a snapshot.
    ///
    /// An error can occur if an event counter cannot be read.
    fn read_snapshot(&mut self) -> Result<Snapshot> {
        let counters_metrics = self
            .counters
            .iter_mut()
            .map(|(event, counter)| {
                let value = counter
                    .read()
                    .map_err(|_| PerfEventError::ErrorReadingCounter(*event))?;
                Ok((*event, value))
            })
            .collect::<Result<HashMap<_, _>>>()?;

        let global_counters_metrics = self
            .global_counters
            .iter_mut()
            .map(|(cpu, counter)| {
                let value = counter
                    .read()
                    .map_err(|_| PerfEventError::ErrorReadingCounter(Event::CpuCycles))?;
                Ok((*cpu, value))
            })
            .collect::<Result<HashMap<_, _>>>()?;

        Ok(Snapshot {
            counters_metrics,
            global_counters_metrics,
        })
    }
}

impl PerfEventCounters {
    pub fn new() -> Self {
        Self::default()
    }
}
