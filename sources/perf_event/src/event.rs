use std::fmt::Display;

use perf_event::events::Hardware;

/// Hardware performance counter event types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Event {
    CpuCycles,
    Instructions,
    CacheMisses,
    BranchMisses,
}

/// Default hardware events to be used in `perf_event`.
pub static EVENTS: &[Event] = &[
    Event::CpuCycles,
    Event::Instructions,
    Event::CacheMisses,
    Event::BranchMisses,
];

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn events_slice_contains_all_variants() {
        assert!(EVENTS.contains(&Event::CpuCycles));
        assert!(EVENTS.contains(&Event::Instructions));
        assert!(EVENTS.contains(&Event::CacheMisses));
        assert!(EVENTS.contains(&Event::BranchMisses));
    }
}
