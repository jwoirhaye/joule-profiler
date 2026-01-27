use std::time::Duration;

use crate::aggregate::sensor_result::SensorResult;
use crate::sensor::Sensors;
use crate::source::error::IntoMetricSourceError;
use crate::source::types::{RawIteration, RawPhase, SourceEvent};
use crate::source::{MetricReader, MetricSource, MetricSourceError};
use log::{debug, trace, warn};
use tokio::{select, sync::mpsc::Receiver, time::Instant};

/// Accumulates metrics from a reader and tracks iterations
#[derive(Debug)]
pub(crate) struct MetricAccumulator<R: MetricReader> {
    /// The underlying metric reader
    metric_reader: R,

    /// Completed iterations
    iterations: Vec<RawIteration<R::Type>>,

    /// Current ongoing iteration
    current_iteration: RawIteration<R::Type>,

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
                trace!("Phase counters retrieved");
                self.current_iteration.phases.push(RawPhase {
                    metrics: phase_counters,
                });
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
    pub fn retrieve(self) -> Result<(SensorResult, Box<dyn MetricSource>), MetricSourceError> {
        debug!("Retrieving results (iterations={})", self.iterations.len());

        let iterations = self
            .iterations
            .into_iter()
            .map(|iteration| iteration.into())
            .collect();

        trace!("Resetting {} metric source for reuse", R::get_name());
        let boxed_source = Box::new(MetricAccumulator::new(self.metric_reader));
        let result = SensorResult { iterations };

        Ok((result, boxed_source))
    }

    pub async fn run_worker(
        self,
        rx: Receiver<SourceEvent>,
    ) -> Result<(SensorResult, Box<dyn MetricSource>), MetricSourceError> {
        if self.metric_reader.has_scheduler() {
            self.run_worker_with_scheduler(rx).await
        } else {
            self.run_worker_without_scheduler(rx).await
        }
    }

    /// Return all sensors from the reader
    pub fn get_sensors(&self) -> Result<Sensors, MetricSourceError> {
        trace!("Retrieving sensors from metric reader {}", R::get_name());

        self.metric_reader
            .get_sensors()
            .map_err(IntoMetricSourceError::into_metric_source_error)
    }

    async fn run_worker_with_scheduler(
        mut self,
        mut rx: Receiver<SourceEvent>,
    ) -> Result<(SensorResult, Box<dyn MetricSource>), MetricSourceError> {
        debug!("Running worker {} with scheduler", R::get_name());

        loop {
            select! {
                biased;

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
                }

                res = self.metric_reader.scheduler(), if self.running => {
                    if let Err(err) = res {
                        warn!("Scheduler error: {}", err);
                        return Err(err.into_metric_source_error());
                    }
                }
            }
        }
    }

    async fn run_worker_without_scheduler(
        mut self,
        mut rx: Receiver<SourceEvent>,
    ) -> Result<(SensorResult, Box<dyn MetricSource>), MetricSourceError> {
        debug!("Running worker {} without scheduler", R::get_name());

        loop {
            if let Some(event) = rx.recv().await {
                trace!("Worker received event: {:?}", event);
                match event {
                    SourceEvent::Measure => self.measure()?,
                    SourceEvent::NewPhase => self.new_phase()?,
                    SourceEvent::NewIteration => self.new_iteration()?,
                    SourceEvent::JoinWorker => return self.retrieve(),
                    _ => {}
                }
            }
        }
    }
}
