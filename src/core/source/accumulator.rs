use std::time::Duration;

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

impl<T: MetricReader> MetricAccumulator<T> {
    /// Create a new accumulator for the given reader
    pub fn new(reader: T) -> Self {
        Self {
            metric_reader: reader,
            iterations: Vec::new(),
            current_iteration: RawIteration::default(),
            last_instant: None,
            poll_count: 0,
            running: false,
        }
    }

    /// Measure the sensors metrics
    pub fn measure(&mut self) -> Result<(), MetricSourceError> {
        let now = Instant::now();
        if let Some(last) = self.last_instant {
            self.current_iteration.total_elapsed += now.duration_since(last);
        }

        self.last_instant = Some(now);
        self.metric_reader
            .measure()
            .map_err(IntoMetricSourceError::into_metric_source_error)?;

        Ok(())
    }

    /// Initialize a new measure phase
    pub fn new_phase(&mut self) -> Result<(), MetricSourceError> {
        if let Ok(phase_counters) = self.metric_reader.retrieve() {
            let phase_counters = RawPhase::new(phase_counters);
            self.current_iteration.phases.push(phase_counters);

            Ok(())
        } else {
            Err(MetricSourceError::ErrorRetrievingCounters)
        }
    }

    /// Initialize a new iteration
    pub fn new_iteration(&mut self) -> Result<(), MetricSourceError> {
        if !self.current_iteration.phases.is_empty() {
            let mut iteration = std::mem::take(&mut self.current_iteration);
            iteration.measure_count = self.poll_count;
            self.poll_count = 0;

            self.current_iteration.total_elapsed = Duration::ZERO;
            self.last_instant = None;
            self.iterations.push(iteration);

            Ok(())
        } else {
            Err(MetricSourceError::NoPhaseInIteration)
        }
    }

    /// Retrieve all sensors measures and return the metric reader
    pub fn retrieve(self) -> Result<(SensorResult, Box<dyn MetricSource>), MetricSourceError> {
        let iterations = self
            .iterations
            .into_iter()
            .map(|iteration| iteration.into())
            .collect();

        let result = SensorResult::new(iterations);
        let boxed_source = Box::new(MetricAccumulator::new(self.metric_reader));
        Ok((result, boxed_source))
    }

    /// Start a worker to process events and measure metrics
    pub async fn run_worker(
        mut self,
        mut rx: Receiver<SourceEvent>,
    ) -> Result<(SensorResult, Box<dyn MetricSource>), MetricSourceError> {
        loop {
            select! {
                Some(event) = rx.recv() => {
                    match event {
                        SourceEvent::Measure => self.measure()?,
                        SourceEvent::NewPhase => self.new_phase()?,
                        SourceEvent::NewIteration => self.new_iteration()?,
                        SourceEvent::StartScheduler => self.running = true,
                        SourceEvent::StopScheduler => self.running = false,
                        SourceEvent::JoinWorker => return self.retrieve(),
                    }
                },
                res =  self.metric_reader.scheduler(), if self.running => {
                    self.poll_count += 1;

                    if let Err(err) = res {
                        return Err(err.into_metric_source_error());
                    }
                }
            }
        }
    }

    /// Return all sensors from the reader
    pub fn get_sensors(&self) -> Result<Sensors, MetricSourceError> {
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
