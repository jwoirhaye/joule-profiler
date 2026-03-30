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

/// Orchestrate a metric source and handle the conversion between raw source results to metrics.
pub struct MetricSourceRuntime<R: MetricReader> {
    accumulator: MetricAccumulator<R>,
    source: R,
}

impl<R: MetricReader> MetricSourceRuntime<R> {
    /// Initialize a [`MetricSourceRuntime`] with the given [`MetricReader`] generic type.
    pub fn new(reader: R) -> Self {
        debug!("Creating MetricAccumulator for reader: {}", R::get_name());

        Self {
            accumulator: MetricAccumulator::new(),
            source: reader,
        }
    }

    /// Runs the worker responsible for source and accumulator management.
    ///
    /// It listens for events through a channel and execute them.
    ///
    /// The `pid_arc` refers to the pid of the profiled program, shared amongst all metric sources.
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

        let result = self.retrieve()?;
        Ok((result, self.source.into()))
    }

    /// Make a measurement.
    #[inline]
    async fn measure_source(&mut self) -> Result<(), MetricSourceError> {
        self.source
            .measure()
            .await
            .map_err(IntoMetricSourceError::into_metric_source_error)
    }

    /// Resets the source counters.
    #[inline]
    async fn reset_source_counters(&mut self) -> Result<(), MetricSourceError> {
        self.source
            .reset()
            .await
            .map_err(IntoMetricSourceError::into_metric_source_error)
    }

    /// Init the source with the profiled program pid.
    #[inline]
    async fn init_source(&mut self, pid: i32) -> Result<(), MetricSourceError> {
        self.source
            .init(pid)
            .await
            .map_err(IntoMetricSourceError::into_metric_source_error)
    }

    /// Initialize a new phase.
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

    /// Initialize a new iteration.
    #[inline]
    fn init_new_iteration(&mut self) -> Result<(), MetricSourceError> {
        self.accumulator.new_iteration()
    }

    /// Retrieve the results from the accumulator and convert them into metrics.
    #[inline]
    fn retrieve(&mut self) -> Result<SensorResult, MetricSourceError> {
        let result = self
            .accumulator
            .retrieve()
            .into_iter()
            .map(|iteration| {
                let phases = iteration
                    .phases
                    .into_iter()
                    .map(|phase| {
                        Ok(SensorPhase {
                            metrics: self
                                .source
                                .to_metrics(phase.metrics)
                                .map_err(IntoMetricSourceError::into_metric_source_error)?,
                        })
                    })
                    .collect::<Result<Vec<_>, MetricSourceError>>()?;

                Ok(SensorIteration { phases })
            })
            .collect::<Result<Vec<_>, MetricSourceError>>()?;

        Ok(SensorResult { iterations: result })
    }

    /// Retrieve source sensors.
    #[inline]
    pub fn get_source_sensors(&self) -> Result<Sensors, MetricSourceError> {
        self.source
            .get_sensors()
            .map_err(IntoMetricSourceError::into_metric_source_error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aggregate::Metrics;
    use crate::sensor::Sensors;
    use crate::source::MetricReader;
    use mockall::mock;
    use std::sync::{Arc, Mutex};
    use tokio::sync::mpsc;

    #[derive(Debug)]
    pub struct MockError(String);

    impl std::fmt::Display for MockError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }
    impl std::error::Error for MockError {}

    mock! {
        pub MetricReader {}
        impl MetricReader for MetricReader {
            type Type  = ();
            type Error = MockError;
            async fn init(&mut self, pid: i32) -> Result<(), MockError>;
            async fn join(&mut self) -> Result<(), MockError>;
            async fn measure(&mut self) -> Result<(), MockError>;
            async fn reset(&mut self) -> Result<(), MockError>;
            async fn retrieve(&mut self) -> Result<(), MockError>;
            fn get_sensors(&self) -> Result<Sensors, MockError>;
            fn to_metrics(&self, v: ()) -> Result<Metrics, MockError>;
            fn get_name() -> &'static str;
        }
    }

    #[derive(Debug, Default)]
    struct Counts {
        init: usize,
        join: usize,
        measure: usize,
        reset: usize,
        retrieve: usize,
    }

    fn mock_reader_counted() -> (MockMetricReader, Arc<Mutex<Counts>>) {
        let counts = Arc::new(Mutex::new(Counts::default()));
        let mut m = MockMetricReader::new();

        let c = counts.clone();
        m.expect_init().returning(move |_| {
            c.lock().unwrap().init += 1;
            Ok(())
        });

        let c = counts.clone();
        m.expect_join().returning(move || {
            c.lock().unwrap().join += 1;
            Ok(())
        });

        let c = counts.clone();
        m.expect_measure().returning(move || {
            c.lock().unwrap().measure += 1;
            Ok(())
        });

        let c = counts.clone();
        m.expect_reset().returning(move || {
            c.lock().unwrap().reset += 1;
            Ok(())
        });

        let c = counts.clone();
        m.expect_retrieve().returning(move || {
            c.lock().unwrap().retrieve += 1;
            Ok(())
        });

        m.expect_get_sensors().returning(|| Ok(vec![]));
        m.expect_to_metrics().returning(|_| Ok(Metrics::default()));

        (m, counts)
    }

    fn pid(value: i32) -> Arc<AtomicI32> {
        Arc::new(AtomicI32::new(value))
    }

    #[tokio::test]
    async fn run_worker_measure_event_calls_measure() {
        let (reader, counts) = mock_reader_counted();
        let rt = MetricSourceRuntime::new(reader);
        let (tx, rx) = mpsc::channel(16);

        tx.send(SourceEvent::Measure).await.unwrap();
        tx.send(SourceEvent::Measure).await.unwrap();
        tx.send(SourceEvent::JoinWorker).await.unwrap();

        rt.run_worker(rx, pid(0)).await.unwrap();

        assert_eq!(counts.lock().unwrap().measure, 2);
    }

    #[tokio::test]
    async fn run_worker_reset_event_calls_reset() {
        let (reader, counts) = mock_reader_counted();
        let rt = MetricSourceRuntime::new(reader);
        let (tx, rx) = mpsc::channel(16);

        tx.send(SourceEvent::Reset).await.unwrap();
        tx.send(SourceEvent::JoinWorker).await.unwrap();

        rt.run_worker(rx, pid(0)).await.unwrap();

        assert_eq!(counts.lock().unwrap().reset, 1);
    }

    #[tokio::test]
    async fn run_worker_init_event_passes_pid() {
        let (reader, counts) = mock_reader_counted();
        let rt = MetricSourceRuntime::new(reader);
        let (tx, rx) = mpsc::channel(16);
        let shared_pid = pid(42);

        tx.send(SourceEvent::Init).await.unwrap();
        tx.send(SourceEvent::JoinWorker).await.unwrap();

        rt.run_worker(rx, shared_pid).await.unwrap();

        assert_eq!(counts.lock().unwrap().init, 1);
    }

    #[tokio::test]
    async fn run_worker_new_phase_calls_retrieve() {
        let (reader, counts) = mock_reader_counted();
        let rt = MetricSourceRuntime::new(reader);
        let (tx, rx) = mpsc::channel(16);

        tx.send(SourceEvent::NewPhase).await.unwrap();
        tx.send(SourceEvent::JoinWorker).await.unwrap();

        rt.run_worker(rx, pid(0)).await.unwrap();

        assert_eq!(counts.lock().unwrap().retrieve, 1);
    }

    #[tokio::test]
    async fn run_worker_join_calls_join_on_source() {
        let (reader, counts) = mock_reader_counted();
        let rt = MetricSourceRuntime::new(reader);
        let (tx, rx) = mpsc::channel(16);

        tx.send(SourceEvent::JoinWorker).await.unwrap();
        rt.run_worker(rx, pid(0)).await.unwrap();

        assert_eq!(counts.lock().unwrap().join, 1);
    }

    #[tokio::test]
    async fn run_worker_measure_error_propagates() {
        let mut reader = MockMetricReader::new();

        reader
            .expect_measure()
            .returning(|| Err(MockError("injected".into())));
        let rt = MetricSourceRuntime::new(reader);
        let (tx, rx) = mpsc::channel(16);

        tx.send(SourceEvent::Measure).await.unwrap();
        tx.send(SourceEvent::JoinWorker).await.unwrap();

        assert!(rt.run_worker(rx, pid(0)).await.is_err());
    }

    #[tokio::test]
    async fn run_worker_full_lifecycle() {
        let (reader, counts) = mock_reader_counted();
        let rt = MetricSourceRuntime::new(reader);
        let (tx, rx) = mpsc::channel(16);

        tx.send(SourceEvent::Measure).await.unwrap();
        tx.send(SourceEvent::Measure).await.unwrap();
        tx.send(SourceEvent::NewPhase).await.unwrap();
        tx.send(SourceEvent::NewIteration).await.unwrap();
        tx.send(SourceEvent::JoinWorker).await.unwrap();

        assert!(rt.run_worker(rx, pid(0)).await.is_ok());

        let c = counts.lock().unwrap();
        assert_eq!(c.measure, 2);
        assert_eq!(c.retrieve, 1);
        assert_eq!(c.join, 1);
    }
}
