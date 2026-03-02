use std::collections::HashMap;

use crate::event::Event;

/// Snapshot of perf_event counters
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Snapshot {
    pub metrics: HashMap<Event, u64>,
}

#[derive(Debug, Clone, Default)]
pub struct Phase {
    pub begin: Snapshot,
    pub end: Snapshot,
}

impl Phase {
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
                    .map(|&prev| current_value.saturating_sub(prev))
                    .unwrap_or(current_value);

                (*event, delta)
            })
            .collect();

        Snapshot { metrics }
    }
}
