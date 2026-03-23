use std::fmt::Display;

use joule_profiler_core::unit::{MetricUnit, Unit, UnitPrefix};
use perf_event::events::Hardware;

/// Hardware performance counter event types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Event {
    CpuCycles,
    Instructions,
    CacheMisses,
    BranchMisses,
}

/// Default hardware events to be used in perf_event.
pub static EVENTS: &[Event] = &[
    Event::CpuCycles,
    Event::Instructions,
    Event::CacheMisses,
    Event::BranchMisses,
];

impl Event {
    /// Returns the unit for this event.
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

#[cfg(test)]
mod tests {
    use super::*;
    use perf_event::events::Hardware;

    #[test]
    fn display_cpu_cycles() {
        assert_eq!(Event::CpuCycles.to_string(), "CPU_CYCLES");
    }

    #[test]
    fn display_instructions() {
        assert_eq!(Event::Instructions.to_string(), "INSTRUCTIONS");
    }

    #[test]
    fn display_cache_misses() {
        assert_eq!(Event::CacheMisses.to_string(), "CACHE_MISSES");
    }

    #[test]
    fn display_branch_misses() {
        assert_eq!(Event::BranchMisses.to_string(), "BRANCH_MISSES");
    }

    #[test]
    fn into_hardware_cpu_cycles() {
        assert_eq!(Hardware::from(Event::CpuCycles), Hardware::CPU_CYCLES);
    }

    #[test]
    fn into_hardware_instructions() {
        assert_eq!(Hardware::from(Event::Instructions), Hardware::INSTRUCTIONS);
    }

    #[test]
    fn into_hardware_cache_misses() {
        assert_eq!(Hardware::from(Event::CacheMisses), Hardware::CACHE_MISSES);
    }

    #[test]
    fn into_hardware_branch_misses() {
        assert_eq!(Hardware::from(Event::BranchMisses), Hardware::BRANCH_MISSES);
    }

    #[test]
    fn unit_is_dimensionless_count_for_all_events() {
        let expected = MetricUnit {
            prefix: UnitPrefix::None,
            unit: Unit::Count,
        };
        for event in EVENTS {
            assert_eq!(event.unit(), expected, "unexpected unit for {event}");
        }
    }

    #[test]
    fn events_slice_contains_all_variants() {
        assert!(EVENTS.contains(&Event::CpuCycles));
        assert!(EVENTS.contains(&Event::Instructions));
        assert!(EVENTS.contains(&Event::CacheMisses));
        assert!(EVENTS.contains(&Event::BranchMisses));
        assert_eq!(EVENTS.len(), 4);
    }
}
