use crate::source::types::{RawPhase};
use crate::source::{MetricReader};
use log::{debug, trace};

/// Accumulates metrics from a reader and tracks phases.
#[derive(Debug)]
pub struct MetricAccumulator<R: MetricReader> {
    /// Already completed iterations.
    phases: Vec<RawPhase<R::Type>>,
}

impl<R: MetricReader> MetricAccumulator<R> {
    /// Create a new accumulator for the given reader.
    pub fn new() -> Self {
        debug!("Creating MetricAccumulator for reader: {}", R::get_name());
        Self::default()
    }

    /// Initialize a new measure phase.
    pub fn new_phase(&mut self, snapshot: R::Type) {
        debug!(
            "Starting new phase (current phases: {})",
            self.phases.len()
        );

        trace!("Phase counters retrieved");
        self.phases.push(RawPhase { metrics: snapshot });
    }

    /// Retrieve all sensors measures.
    pub fn retrieve(&mut self) -> Vec<RawPhase<R::Type>> {
        debug!("Retrieving results (iterations={})", self.phases.len());
        std::mem::take(&mut self.phases)
    }
}

impl<R: MetricReader> Default for MetricAccumulator<R> {
    fn default() -> Self {
        Self {
            phases: Vec::default(),
        }
    }
}
