use std::collections::HashMap;

use crate::event::Event;

/// Snapshot of `perf_event` counters.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Snapshot {
    pub metrics: HashMap<Event, u64>,
}

/// A pair of snapshots delimiting a phase.
#[derive(Debug, Clone, Default)]
pub struct Phase {
    /// The snapshot made at the start of a phase.
    pub begin: Snapshot,

    /// End snapshot of the phase.
    pub end: Snapshot,
}

impl Phase {
    /// Computes the per-event delta between begin and end.
    pub fn diff(&self) -> Snapshot {
        let metrics = self
            .end
            .metrics
            .iter()
            .map(|(event, &current_value)| {
                let delta = self
                    .begin
                    .metrics
                    .get(event)
                    .map_or(current_value, |&prev| current_value.wrapping_sub(prev));

                (*event, delta)
            })
            .collect();

        Snapshot { metrics }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::Event;

    fn snapshot(metrics: Vec<(Event, u64)>) -> Snapshot {
        Snapshot {
            metrics: metrics.into_iter().collect(),
        }
    }

    #[test]
    fn diff_basic_delta() {
        let phase = Phase {
            begin: snapshot(vec![(Event::CpuCycles, 100)]),
            end: snapshot(vec![(Event::CpuCycles, 350)]),
        };
        assert_eq!(phase.diff(), snapshot(vec![(Event::CpuCycles, 250)]));
    }

    #[test]
    fn diff_multiple_events() {
        let phase = Phase {
            begin: snapshot(vec![(Event::CpuCycles, 100), (Event::Instructions, 200)]),
            end: snapshot(vec![(Event::CpuCycles, 400), (Event::Instructions, 500)]),
        };
        let diff = phase.diff();
        assert_eq!(diff.metrics[&Event::CpuCycles], 300);
        assert_eq!(diff.metrics[&Event::Instructions], 300);
    }

    #[test]
    fn diff_equal_values_returns_zero() {
        let phase = Phase {
            begin: snapshot(vec![(Event::CpuCycles, 42)]),
            end: snapshot(vec![(Event::CpuCycles, 42)]),
        };
        assert_eq!(phase.diff(), snapshot(vec![(Event::CpuCycles, 0)]));
    }

    #[test]
    fn diff_wraps_on_counter_overflow() {
        let phase = Phase {
            begin: snapshot(vec![(Event::CpuCycles, u64::MAX - 5)]),
            end: snapshot(vec![(Event::CpuCycles, 10)]),
        };
        assert_eq!(phase.diff(), snapshot(vec![(Event::CpuCycles, 16)]));
    }

    #[test]
    fn diff_event_missing_in_begin_uses_end_value() {
        let phase = Phase {
            begin: snapshot(vec![]),
            end: snapshot(vec![(Event::CacheMisses, 77)]),
        };
        assert_eq!(phase.diff(), snapshot(vec![(Event::CacheMisses, 77)]));
    }

    #[test]
    fn diff_event_missing_in_end_is_absent_from_result() {
        let phase = Phase {
            begin: snapshot(vec![(Event::CpuCycles, 100), (Event::Instructions, 200)]),
            end: snapshot(vec![(Event::CpuCycles, 150)]),
        };
        let diff = phase.diff();
        assert_eq!(diff.metrics.len(), 1);
        assert_eq!(diff.metrics[&Event::CpuCycles], 50);
        assert!(!diff.metrics.contains_key(&Event::Instructions));
    }

    #[test]
    fn diff_empty_snapshots_returns_empty() {
        let phase = Phase::default();
        assert_eq!(phase.diff(), Snapshot::default());
    }
}
