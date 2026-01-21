use std::time::Duration;

use log::{debug, trace, warn};
use tokio::{select, sync::mpsc::Receiver, time::Instant};

use crate::core::{
    aggregate::sensor_result::SensorResult,
    sensor::Sensors,
    source::{
        MetricSource,
        error::MetricSourceError,
        reader::MetricReader,
        types::{RawIteration, RawPhase, SourceEvent},
    },
};

/// Accumulates metrics from a reader and tracks iterations
#[derive(Debug)]
pub(crate) struct MetricAccumulator<R: MetricReader> {
    /// The underlying metric reader
    metric_reader: R,

    /// Completed iterations
    iterations: Vec<RawIteration<R::Type>>,

    /// Current ongoing iteration
    current_iteration: RawIteration<R::Type>,

    /// Count of polling measurements
    poll_count: u64,

    /// Monotonic timestamp of last snapshot
    last_instant: Option<Instant>,

    running: bool,
}

impl<R: MetricReader> MetricAccumulator<R> {
    /// Create a new accumulator for the given reader
    pub fn new(reader: R) -> Self {
        debug!("Creating MetricAccumulator for reader: {}", R::get_name());

        Self {
            metric_reader: reader,
            iterations: Vec::new(),
            current_iteration: RawIteration::default(),
            last_instant: None,
            poll_count: 0,
            running: false,
        }
    }

    pub fn measure(&mut self) -> Result<(), MetricSourceError> {
        let now = Instant::now();

        if let Some(last) = self.last_instant {
            let delta = now.duration_since(last);
            self.current_iteration.total_elapsed += delta;

            trace!("Measure delta: {:?}", delta);
        } else {
            trace!("First measure in iteration");
        }

        self.last_instant = Some(now);
        self.metric_reader
            .measure()
            .map_err(IntoMetricSourceError::into_metric_source_error)?;

        Ok(())
    }

    /// Initialize a new measure phase
    pub fn new_phase(&mut self) -> Result<(), MetricSourceError> {
        debug!(
            "Starting new phase (current phases: {})",
            self.current_iteration.phases.len()
        );

        match self.metric_reader.retrieve() {
            Ok(phase_counters) => {
                let phase = RawPhase::new(phase_counters);
                trace!("Phase counters retrieved");
                self.current_iteration.phases.push(phase);
                Ok(())
            }
            Err(_) => {
                warn!("Failed to retrieve counters for new phase");
                Err(MetricSourceError::ErrorRetrievingCounters)
            }
        }
    }

    /// Initialize a new iteration
    pub fn new_iteration(&mut self) -> Result<(), MetricSourceError> {
        debug!(
            "Finalizing iteration (phases={}, polls={})",
            self.current_iteration.phases.len(),
            self.poll_count
        );

        if !self.current_iteration.phases.is_empty() {
            let mut iteration = std::mem::take(&mut self.current_iteration);
            iteration.poll_count = self.poll_count;

            trace!("Iteration total elapsed: {:?}", iteration.total_elapsed);

            self.poll_count = 0;
            self.current_iteration.total_elapsed = Duration::ZERO;
            self.last_instant = None;

            self.iterations.push(iteration);
            Ok(())
        } else {
            warn!("Attempted to create iteration with no phases");
            Err(MetricSourceError::NoPhaseInIteration)
        }
    }

    /// Retrieve all sensors measures and return the metric reader
    pub fn retrieve(self) -> Result<(SensorResult, Box<dyn MetricSource>), MetricSourceError> {
        debug!("Retrieving results (iterations={})", self.iterations.len());

        let iterations = self
            .iterations
            .into_iter()
            .map(|iteration| iteration.into())
            .collect();

        let result = SensorResult::new(iterations);

        trace!("Resetting {} metric source for reuse", R::get_name());
        let boxed_source = Box::new(MetricAccumulator::new(self.metric_reader));

        Ok((result, boxed_source))
    }

    /// Start a worker to process events and measure metrics
    pub async fn run_worker(
        mut self,
        mut rx: Receiver<SourceEvent>,
    ) -> Result<(SensorResult, Box<dyn MetricSource>), MetricSourceError> {
        debug!("MetricAccumulator worker started");

        loop {
            select! {
                Some(event) = rx.recv() => {
                    trace!("Worker received event: {:?}", event);

                    match event {
                        SourceEvent::Measure => self.measure()?,
                        SourceEvent::NewPhase => self.new_phase()?,
                        SourceEvent::NewIteration => self.new_iteration()?,
                        SourceEvent::StartScheduler => {
                            debug!("Scheduler started");
                            self.running = true;
                        }
                        SourceEvent::StopScheduler => {
                            debug!("Scheduler stopped");
                            self.running = false;
                        }
                        SourceEvent::JoinWorker => {
                            debug!("Worker join requested");
                            return self.retrieve();
                        }
                    }
                },

                res = self.metric_reader.scheduler(), if self.running => {
                    self.poll_count += 1;
                    trace!("Scheduler tick (poll #{})", self.poll_count);

                    if let Err(err) = res {
                        warn!("Scheduler error: {}", err);
                        return Err(err.into_metric_source_error());
                    }
                }
            }
        }
    }

    /// Return all sensors from the reader
    pub fn get_sensors(&self) -> Result<Sensors, MetricSourceError> {
        trace!("Retrieving sensors from metric reader {}", R::get_name());

        self.metric_reader
            .get_sensors()
            .map_err(IntoMetricSourceError::into_metric_source_error)
    }
}

pub trait IntoMetricSourceError {
    fn into_metric_source_error(self) -> MetricSourceError;
}

impl<T> IntoMetricSourceError for T
where
    T: std::error::Error + Send + Sync + 'static,
{
    fn into_metric_source_error(self) -> MetricSourceError {
        MetricSourceError::External(Box::new(self))
    }
}
