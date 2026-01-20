use std::time::Duration;

use tokio::{select, sync::mpsc::Receiver, time::Instant};

use crate::core::{
    phase::SourcePhase,
    sensor::Sensors,
    source::{
        MetricSource, SourceIteration, error::MetricSourceError, reader::MetricReader,
        result::SensorResult, types::SourceEvent,
    },
};

#[derive(Debug)]
pub struct MetricAccumulator<R: MetricReader> {
    metric_reader: R,

    iterations: Vec<SourceIteration<R::Type>>,

    current_iteration: SourceIteration<R::Type>,

    /// Monotonic timestamp of last snapshot
    last_instant: Option<Instant>,
}

impl<T: MetricReader> MetricAccumulator<T> {
    pub fn new(reader: T) -> Self {
        Self {
            metric_reader: reader,
            iterations: Vec::new(),
            current_iteration: SourceIteration::default(),
            last_instant: None,
        }
    }

    /// Measure the sensors metrics.
    pub fn measure(&mut self) -> Result<(), MetricSourceError> {
        let now = Instant::now();
        if let Some(last) = self.last_instant {
            self.current_iteration.total_elapsed += now.duration_since(last);
        }

        self.last_instant = Some(now);
        self.current_iteration.measure_count += 1;
        self.metric_reader.measure().map_err(|err| err.into())?;

        Ok(())
    }

    /// Initialize a new measure phase.
    pub fn new_phase(&mut self) -> Result<(), MetricSourceError> {
        if let Ok((counters, measure_count)) = self.metric_reader.retrieve_counters() {
            let phase_counters = SourcePhase::new(counters);
            self.current_iteration.phases.push(phase_counters);
    
            Ok(())
        } else {
            Err(MetricSourceError::ErrorRetrievingCounters)
        }
    }

    /// Initialize a new iteration.
    pub fn new_iteration(&mut self) -> Result<(), MetricSourceError> {
        if !self.current_iteration.phases.is_empty() {
            let iteration = std::mem::take(&mut self.current_iteration);

            self.current_iteration.measure_count = 0;
            self.current_iteration.total_elapsed = Duration::ZERO;
            self.last_instant = None;
            self.iterations.push(iteration);

            Ok(())
        } else {
            Err(MetricSourceError::NoPhaseInIteration)
        }
    }

    fn set_polling(&mut self, polling: bool) -> Result<(), MetricSourceError> {
        self.metric_reader
            .set_polling(polling)
            .map_err(|err| err.into())
    }

    /// Retrieve all sensors measures.
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

    /// Start a worker thread to measure the source.
    pub async fn run_worker(
        mut self,
        mut rx: Receiver<SourceEvent>,
    ) -> Result<(SensorResult, Box<dyn MetricSource>), MetricSourceError> {
        loop {
            select! {
                Err(err) = self.metric_reader.internal_scheduler() => {
                    return Err(err.into());
                }
                Some(event) = rx.recv() => {
                    match event {
                        SourceEvent::Measure => self.measure()?,
                        SourceEvent::NewPhase => self.new_phase()?,
                        SourceEvent::NewIteration => self.new_iteration()?,
                        SourceEvent::StartPolling => self.set_polling(true)?,
                        SourceEvent::StopPolling => self.set_polling(false)?,
                        SourceEvent::JoinWorker => return self.retrieve(),
                    }
                },
            }
        }
    }

    pub fn get_sensors(&self) -> Result<Sensors, MetricSourceError> {
        self.metric_reader.get_sensors().map_err(|err| err.into())
    }
}
