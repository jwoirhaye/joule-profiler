use std::fmt::Display;

use joule_profiler_core::unit::{MetricUnit, Unit, UnitPrefix};
use perf_event::events::Hardware;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Event {
    CpuCycles,
    Instructions,
    CacheMisses,
    BranchMisses,
}

pub static EVENTS: &[Event] = &[
    Event::CpuCycles,
    Event::Instructions,
    Event::CacheMisses,
    Event::BranchMisses,
];

impl Event {
    pub const fn unit(&self) -> MetricUnit {
        MetricUnit {
            prefix: UnitPrefix::None,
            unit: Unit::Count,
        }
    }
}

impl From<Event> for Hardware {
    fn from(event: Event) -> Self {
        match event {
            Event::CpuCycles => Hardware::CPU_CYCLES,
            Event::Instructions => Hardware::INSTRUCTIONS,
            Event::CacheMisses => Hardware::CACHE_MISSES,
            Event::BranchMisses => Hardware::BRANCH_MISSES,
        }
    }
}

impl Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Event::CpuCycles => "CPU_CYCLES",
            Event::Instructions => "INSTRUCTIONS",
            Event::CacheMisses => "CACHE_MISSES",
            Event::BranchMisses => "BRANCH_MISSES",
        })
    }
}
