use crate::source::types::{RawIteration, RawPhase};
use crate::source::{MetricReader, MetricSourceError};
use log::{debug, error, trace};

/// Accumulates metrics from a reader and tracks iterations.
#[derive(Debug)]
pub struct MetricAccumulator<R: MetricReader> {
    /// Already completed iterations.
    iterations: Vec<RawIteration<R::Type>>,

    /// Current ongoing iteration.
    current_iteration: RawIteration<R::Type>,
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
            self.current_iteration.phases.len()
        );

        trace!("Phase counters retrieved");
        self.current_iteration
            .phases
            .push(RawPhase { metrics: snapshot });
    }

    /// Initialize a new iteration.
    pub fn new_iteration(&mut self) -> Result<(), MetricSourceError> {
        if !self.current_iteration.phases.is_empty() {
            self.iterations
                .push(std::mem::take(&mut self.current_iteration));
            Ok(())
        } else {
            error!("Attempted to create iteration with no phases");
            Err(MetricSourceError::NoPhaseInIterationError)
        }
    }

    /// Retrieve all sensors measures.
    pub fn retrieve(&mut self) -> Vec<RawIteration<R::Type>> {
        debug!("Retrieving results (iterations={})", self.iterations.len());
        std::mem::take(&mut self.iterations)
    }
}

impl<R: MetricReader> Default for MetricAccumulator<R> {
    fn default() -> Self {
        Self {
            iterations: Default::default(),
            current_iteration: Default::default(),
        }
    }
}
