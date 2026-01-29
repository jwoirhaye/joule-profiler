use std::marker::PhantomData;
use std::time::Duration;

use crate::aggregate::sensor_result::SensorResult;
use crate::source::types::{RawIteration, RawPhase};
use crate::source::{MetricReader, MetricSourceError};
use log::{debug, trace, warn};
use tokio::time::Instant;

/// Accumulates metrics from a reader and tracks iterations
#[derive(Debug)]
pub struct MetricAccumulator<R: MetricReader> {
    /// The underlying metric reader
    metric_reader: PhantomData<R>,

    /// Completed iterations
    iterations: Vec<RawIteration<R::Type>>,

    /// Current ongoing iteration
    current_iteration: RawIteration<R::Type>,

    /// Monotonic timestamp of last snapshot
    last_instant: Option<Instant>,
}

impl<R: MetricReader> MetricAccumulator<R> {
    /// Create a new accumulator for the given reader
    pub fn new() -> Self {
        debug!("Creating MetricAccumulator for reader: {}", R::get_name());

        Self {
            metric_reader: PhantomData,
            iterations: Vec::new(),
            current_iteration: RawIteration::default(),
            last_instant: None,
        }
    }

    /// Initialize a new measure phase
    pub fn new_phase(&mut self, snapshot: R::Type) -> Result<(), MetricSourceError> {
        debug!(
            "Starting new phase (current phases: {})",
            self.current_iteration.phases.len()
        );

        trace!("Phase counters retrieved");
        self.current_iteration
            .phases
            .push(RawPhase { metrics: snapshot });
        Ok(())
    }

    /// Initialize a new iteration
    pub fn new_iteration(&mut self) -> Result<(), MetricSourceError> {
        if !self.current_iteration.phases.is_empty() {
            let iteration = std::mem::take(&mut self.current_iteration);

            trace!("Iteration total elapsed: {:?}", iteration.total_elapsed);

            self.current_iteration.total_elapsed = Duration::ZERO;
            self.last_instant = None;

            self.iterations.push(iteration);
            Ok(())
        } else {
            warn!("Attempted to create iteration with no phases");
            Err(MetricSourceError::NoPhaseInIterationError)
        }
    }

    /// Retrieve all sensors measures and return the metric reader
    pub fn retrieve(self) -> Result<SensorResult, MetricSourceError> {
        debug!("Retrieving results (iterations={})", self.iterations.len());

        let iterations = self
            .iterations
            .into_iter()
            .map(|iteration| iteration.into())
            .collect();

        trace!("Resetting {} metric source for reuse", R::get_name());
        Ok(SensorResult { iterations })
    }
}
