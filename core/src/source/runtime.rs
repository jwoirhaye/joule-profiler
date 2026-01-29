use log::debug;
use tokio::{
    sync::mpsc::{Receiver, Sender},
    task::JoinHandle,
};

use crate::{
    aggregate::sensor_result::SensorResult,
    source::{
        MetricReader, MetricSource, MetricSourceError,
        accumulator::MetricAccumulator,
        error::IntoMetricSourceError,
        types::{SourceEvent, SourceEventer},
    },
};

pub struct MetricRuntime<R: MetricReader> {
    pub accumulator: MetricAccumulator<R>,
    pub source: R,
    active: bool,
}

pub type Handle = JoinHandle<Result<(SensorResult, Box<dyn MetricSource>), MetricSourceError>>;

impl<R: MetricReader> MetricRuntime<R> {
    pub fn new(reader: R) -> Self {
        debug!("Creating MetricAccumulator for reader: {}", R::get_name());

        Self {
            accumulator: MetricAccumulator::new(),
            source: reader,
            active: false,
        }
    }

    pub async fn run_worker(
        mut self,
        tx: Sender<SourceEvent>,
        mut rx: Receiver<SourceEvent>,
    ) -> Result<(SensorResult, Box<dyn MetricSource>), MetricSourceError> {
        let source_handle = self
            .source
            .run(SourceEventer::new(tx.clone()))
            .await
            .map_err(IntoMetricSourceError::into_metric_source_error)?;

        loop {
            if let Some(event) = rx.recv().await {
                match event {
                    SourceEvent::Measure => {
                        if self.active {
                            self.measure_source()?
                        }
                    }
                    SourceEvent::NewPhase => self.new_phase()?,
                    SourceEvent::NewIteration => self.new_iteration()?,
                    SourceEvent::JoinWorker => break,
                    SourceEvent::Start => self.active = true,
                    SourceEvent::Stop => self.active = false,
                }
            }
        }

        if let Some(handle) = source_handle {
            handle.abort();
        }

        let result = self.accumulator.retrieve()?;

        Ok((result, self.source.into()))
    }

    fn measure_source(&mut self) -> Result<(), MetricSourceError> {
        self.source
            .measure()
            .map_err(IntoMetricSourceError::into_metric_source_error)
    }

    fn new_phase(&mut self) -> Result<(), MetricSourceError> {
        let result = self
            .source
            .retrieve()
            .map_err(IntoMetricSourceError::into_metric_source_error)?;
        self.accumulator.new_phase(result)?;
        Ok(())
    }

    fn new_iteration(&mut self) -> Result<(), MetricSourceError> {
        self.accumulator.new_iteration()
    }
}
