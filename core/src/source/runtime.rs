use std::sync::{
    Arc,
    atomic::{AtomicI32, Ordering},
};

use log::debug;
use tokio::sync::mpsc::Receiver;

use crate::{
    aggregate::{iteration::SensorIteration, phase::SensorPhase, sensor_result::SensorResult},
    sensor::Sensors,
    source::{
        MetricReader, MetricSource, MetricSourceError, accumulator::MetricAccumulator,
        error::IntoMetricSourceError, types::SourceEvent,
    },
};

/// Orchestrate a metric source and handle the conversion between raw source results to metrics
pub struct MetricSourceRuntime<R: MetricReader> {
    accumulator: MetricAccumulator<R>,
    source: R,
}

impl<R: MetricReader> MetricSourceRuntime<R> {
    /// Initialize a MetricRuntime of the given MetricReader generic type
    pub fn new(reader: R) -> Self {
        debug!("Creating MetricAccumulator for reader: {}", R::get_name());

        Self {
            accumulator: MetricAccumulator::new(),
            source: reader,
        }
    }

    /// Run the worker responsible for source and accumulator management
    ///
    /// The pid_arc refers to the pid of the profiled program, updated on every Init event
    pub async fn run_worker(
        mut self,
        mut rx: Receiver<SourceEvent>,
        pid_arc: Arc<AtomicI32>,
    ) -> Result<(SensorResult, Box<dyn MetricSource>), MetricSourceError> {
        loop {
            if let Some(event) = rx.recv().await {
                match event {
                    SourceEvent::Measure => self.measure_source().await?,
                    SourceEvent::Reset => self.reset_source_counters().await?,
                    SourceEvent::NewPhase => self.init_new_phase().await?,
                    SourceEvent::NewIteration => self.init_new_iteration()?,
                    SourceEvent::Init => {
                        let pid = pid_arc.load(Ordering::Relaxed);
                        self.init_source(pid).await?;
                    }
                    SourceEvent::JoinWorker => break,
                }
            }
        }

        self.source
            .join()
            .await
            .map_err(IntoMetricSourceError::into_metric_source_error)?;

        let result = self.retrieve();
        Ok((result, self.source.into()))
    }

    /// Measure the source and convert the error if any occur into a MetricSourceError
    #[inline]
    async fn measure_source(&mut self) -> Result<(), MetricSourceError> {
        self.source
            .measure()
            .await
            .map_err(IntoMetricSourceError::into_metric_source_error)
    }

    /// Resets the source counters
    #[inline]
    async fn reset_source_counters(&mut self) -> Result<(), MetricSourceError> {
        self.source
            .reset()
            .await
            .map_err(IntoMetricSourceError::into_metric_source_error)
    }

    /// Init the source with the profiled program pid
    #[inline]
    async fn init_source(&mut self, pid: i32) -> Result<(), MetricSourceError> {
        self.source
            .init(pid)
            .await
            .map_err(IntoMetricSourceError::into_metric_source_error)
    }

    /// Initialize a new phase and convert the error if any occur into a MetricSourceError
    #[inline]
    async fn init_new_phase(&mut self) -> Result<(), MetricSourceError> {
        let result = self
            .source
            .retrieve()
            .await
            .map_err(IntoMetricSourceError::into_metric_source_error)?;
        self.accumulator.new_phase(result);
        Ok(())
    }

    /// Initialize a new iteration and convert the error if any occur into a MetricSourceError
    #[inline]
    fn init_new_iteration(&mut self) -> Result<(), MetricSourceError> {
        self.accumulator.new_iteration()
    }

    /// Retrieve the results from the accumulator and convert them into metrics
    #[inline]
    fn retrieve(&mut self) -> SensorResult {
        let result = self
            .accumulator
            .retrieve()
            .into_iter()
            .map(|iteration| {
                let phases = iteration
                    .phases
                    .into_iter()
                    .map(|phase| SensorPhase {
                        metrics: self.source.to_metrics(phase.metrics),
                    })
                    .collect();
                SensorIteration { phases }
            })
            .collect();
        SensorResult { iterations: result }
    }

    /// Retrieve source sensors
    #[inline]
    pub fn get_source_sensors(&self) -> Result<Sensors, MetricSourceError> {
        self.source
            .get_sensors()
            .map_err(IntoMetricSourceError::into_metric_source_error)
    }
}
