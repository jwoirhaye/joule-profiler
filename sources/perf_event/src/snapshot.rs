use std::{collections::HashMap, ops::AddAssign};

use crate::event::Event;

/// Snapshot of perf_event counters
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Snapshot {
    pub metrics: HashMap<Event, u64>,
}

impl Snapshot {
    /// Compute the delta between two snapshots (self - previous).
    ///
    /// Returns a new Snapshot with the difference in counter values.
    pub fn delta(&self, previous: &Snapshot) -> Snapshot {
        let metrics = self
            .metrics
            .iter()
            .map(|(event, &current_value)| {
                let delta = previous
                    .metrics
                    .get(event)
                    .map(|&prev| current_value.saturating_sub(prev))
                    .unwrap_or(current_value);

                (*event, delta)
            })
            .collect();

        Snapshot { metrics }
    }
}

impl AddAssign for Snapshot {
    fn add_assign(&mut self, rhs: Snapshot) {
        for (event, value) in rhs.metrics {
            self.metrics
                .entry(event)
                .and_modify(|total| *total += value)
                .or_insert(value);
        }
    }
}
