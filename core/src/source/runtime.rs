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
    pub async fn run_worker(
        mut self,
        mut rx: Receiver<SourceEvent>,
    ) -> Result<(SensorResult, Box<dyn MetricSource>), MetricSourceError> {
        self.source
            .init()
            .await
            .map_err(IntoMetricSourceError::into_metric_source_error)?;

        loop {
            if let Some(event) = rx.recv().await {
                match event {
                    SourceEvent::Measure => self.measure_source().await?,
                    SourceEvent::NewPhase => self.new_phase().await?,
                    SourceEvent::NewIteration => self.new_iteration()?,
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
    async fn measure_source(&mut self) -> Result<(), MetricSourceError> {
        self.source
            .measure()
            .await
            .map_err(IntoMetricSourceError::into_metric_source_error)
    }

    /// Initialize a new phase and convert the error if any occur into a MetricSourceError
    async fn new_phase(&mut self) -> Result<(), MetricSourceError> {
        let result = self
            .source
            .retrieve()
            .await
            .map_err(IntoMetricSourceError::into_metric_source_error)?;
        self.accumulator.new_phase(result);
        Ok(())
    }

    /// Initialize a new iteration and convert the error if any occur into a MetricSourceError
    fn new_iteration(&mut self) -> Result<(), MetricSourceError> {
        self.accumulator.new_iteration()
    }

    /// Retrieve the results from the accumulator and convert them into metrics
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

    pub fn get_source_sensors(&self) -> Result<Sensors, MetricSourceError> {
        self.source
            .get_sensors()
            .map_err(IntoMetricSourceError::into_metric_source_error)
    }
}
