//! Core orchestration module for `JouleProfiler`.
//!
//! This module defines the core logic for metric sources orchestration through [`SourceOrchestrator`] structure.

use std::sync::Arc;
use std::sync::atomic::AtomicI32;

use crate::aggregate::sensor_result::SensorResult;
use crate::orchestrator::error::OrchestratorError;
use crate::source::types::SourceEvent;
use crate::source::{MetricSource, MetricSourceError};
use futures::future::try_join_all;
use tokio::sync::mpsc::error::SendError;
use tokio::{sync::mpsc::Sender, task::JoinHandle};

pub mod error;

/// The handle describing the return type of a source worker.
type Handle = JoinHandle<Result<(SensorResult, Box<dyn MetricSource>), MetricSourceError>>;

/// Orchestrates the metric sources and send them the profiler's messages through asynchronous channels.
/// It is a proxy between the profiler and the sources and is responsible of their lifecycle.
#[derive(Default)]
pub struct SourceOrchestrator {
    /// The event channels sender used to manage the metric sources.
    senders: Vec<Sender<SourceEvent>>,

    /// The handles of the worker tasks, used for joining sources gracefully.
    handles: Vec<Handle>,
}

impl SourceOrchestrator {
    /// Starts all the metric sources.
    ///
    /// The function shares the atomic integer representing the profiled program's pid, used by some sources for per-process profiling (e.g. `perf_event`).
    /// Stores the sources handles and the channels senders to be able to gracefully join the sources and send events.
    #[inline]
    pub fn run(&mut self, sources: Vec<Box<dyn MetricSource>>, shared_pid: &Arc<AtomicI32>) {
        let nb_sources = sources.len();
        let mut senders = Vec::with_capacity(nb_sources);
        let mut handles = Vec::with_capacity(nb_sources);

        for source in sources {
            let (handle, tx) = source.run(shared_pid.clone());
            senders.push(tx);
            handles.push(handle);
        }

        self.handles = handles;
        self.senders = senders;
    }

    /// Measures the metrics of each metric source.
    #[inline]
    pub async fn measure(&mut self) -> Result<(), OrchestratorError> {
        self.send_event(SourceEvent::Measure).await
    }

    /// Resets the counters of each metric source
    #[inline]
    pub async fn reset(&mut self) -> Result<(), OrchestratorError> {
        self.send_event(SourceEvent::Reset).await
    }

    /// Initializes each metric source.
    /// Called when the program execution is stopped to inizialize sources requiring pid filtering (e.g. `perf_event`).
    #[inline]
    pub async fn init(&mut self) -> Result<(), OrchestratorError> {
        self.send_event(SourceEvent::Init).await
    }

    /// Initializes a new phase for each metric source.
    #[inline]
    pub async fn new_phase(&mut self) -> Result<(), OrchestratorError> {
        self.send_event(SourceEvent::NewPhase).await
    }

    /// Initializes a new iteration for each metric source.
    #[inline]
    pub async fn new_iteration(&mut self) -> Result<(), OrchestratorError> {
        self.send_event(SourceEvent::NewIteration).await
    }

    /// Retrieves and merge results from all sources.
    ///
    /// Returns a tuple containing the aggregated results and the list of the metric sources in order to reuse them.
    ///
    /// # Errors
    ///
    /// If not enough snapshots have been made, a [`NotEnoughSnapshots`](`OrchestratorError::NotEnoughSnapshots`) error is returned.
    /// Also if an error has occured in one of the sources, it will be returned.
    pub async fn finalize(
        &mut self,
    ) -> Result<(SensorResult, Vec<Box<dyn MetricSource>>), OrchestratorError> {
        let (results, sources) = self.join_all().await?;
        let merged = SensorResult::merge(results).ok_or(OrchestratorError::NotEnoughSnapshots)?;
        Ok((merged, sources))
    }

    /// Stop the worker thread of each metrics sources to join threads gracefully.
    #[inline]
    async fn join(&mut self) -> Result<(), OrchestratorError> {
        self.send_event(SourceEvent::JoinWorker).await
    }

    /// Sends the provided event to all the metrics sources.
    ///
    /// If an error is encountered in a source, then the worker is aborted and the error is returned.
    async fn send_event(&mut self, event: SourceEvent) -> Result<(), OrchestratorError> {
        let futures: Vec<_> = self
            .senders
            .iter_mut()
            .enumerate()
            .map(|(i, tx)| async move { tx.send(event).await.map_err(|send_err| (i, send_err)) })
            .collect();

        if let Err((failed_index, send_err)) = try_join_all(futures).await {
            Err(self.handle_event_error(failed_index, send_err).await)
        } else {
            Ok(())
        }
    }

    /// Handles the error from a disconnected source (failed) and return it.
    async fn handle_event_error(
        &mut self,
        failed_index: usize,
        err: SendError<SourceEvent>,
    ) -> OrchestratorError {
        if let Some(handle) = self.handles.get_mut(failed_index) {
            match handle.await {
                Ok(Ok((_result, _source))) => err.into(),
                Ok(Err(metric_err)) => metric_err.into(),
                Err(join_err) => join_err.into(),
            }
        } else {
            err.into()
        }
    }

    /// Joins all workers and collect results.
    /// Waits until workers termination.
    /// If an error has occured in one of the sources, it will be returned.
    async fn join_all(
        &mut self,
    ) -> Result<(Vec<SensorResult>, Vec<Box<dyn MetricSource>>), OrchestratorError> {
        self.join().await?;

        let handles = std::mem::take(&mut self.handles);

        let mut results = Vec::with_capacity(handles.len());
        let mut sources = Vec::with_capacity(handles.len());

        for handle in handles {
            let (result, source) = handle.await??;
            results.push(result);
            sources.push(source);
        }

        Ok((results, sources))
    }
}

#[cfg(test)]
mod tests {
    use mockall::mock;

    use crate::{sensor::Sensors, source::MetricReader, types::Metrics};

    use super::*;
    use std::sync::{Mutex, atomic::Ordering};

    #[derive(Debug)]
    pub struct MockError;

    impl std::fmt::Display for MockError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "mock error")
        }
    }

    impl std::error::Error for MockError {}

    mock! {
        pub MetricReader {}

        impl MetricReader for MetricReader {
            type Type = ();
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

    fn pid() -> Arc<AtomicI32> {
        Arc::new(AtomicI32::new(0))
    }

    fn mock_source() -> Box<dyn MetricSource> {
        MockMetricReader::new().into()
    }

    fn mock_source_with_counts() -> (Box<dyn MetricSource>, Arc<Mutex<Counts>>) {
        let (r, counts) = mock_reader_counted();
        (r.into(), counts)
    }

    #[tokio::test]
    async fn finalize_no_sources_returns_not_enough_snapshots() {
        let mut o = SourceOrchestrator::default();
        assert!(matches!(
            o.finalize().await,
            Err(OrchestratorError::NotEnoughSnapshots)
        ));
    }

    #[tokio::test]
    async fn run_registers_one_sender_and_handle_per_source() {
        let mut o = SourceOrchestrator::default();
        o.run(vec![mock_source(), mock_source()], &pid());
        assert_eq!(o.senders.len(), 2);
        assert_eq!(o.handles.len(), 2);
    }

    #[tokio::test]
    async fn run_replaces_previous_sources() {
        let mut o = SourceOrchestrator::default();
        o.run(vec![mock_source(), mock_source()], &pid());
        o.run(vec![mock_source()], &pid());
        assert_eq!(o.senders.len(), 1);
        assert_eq!(o.handles.len(), 1);
    }

    #[tokio::test]
    async fn measure_event_reaches_worker() {
        let (source, counts) = mock_source_with_counts();
        let mut o = SourceOrchestrator::default();
        o.run(vec![source], &pid());

        o.measure().await.unwrap();
        o.measure().await.unwrap();

        let _ = o.finalize().await;
        assert_eq!(counts.lock().unwrap().measure, 2);
    }

    #[tokio::test]
    async fn reset_event_reaches_worker() {
        let (source, counts) = mock_source_with_counts();
        let mut o = SourceOrchestrator::default();
        o.run(vec![source], &pid());

        o.reset().await.unwrap();

        let _ = o.finalize().await;
        assert_eq!(counts.lock().unwrap().reset, 1);
    }

    #[tokio::test]
    async fn init_event_reaches_worker() {
        let (source, counts) = mock_source_with_counts();
        let shared_pid = pid();
        shared_pid.store(42, Ordering::SeqCst);
        let mut o = SourceOrchestrator::default();
        o.run(vec![source], &shared_pid);

        o.init().await.unwrap();

        let _ = o.finalize().await;
        assert_eq!(counts.lock().unwrap().init, 1);
    }

    #[tokio::test]
    async fn new_phase_event_reaches_worker() {
        let (source, counts) = mock_source_with_counts();
        let mut o = SourceOrchestrator::default();
        o.run(vec![source], &pid());

        o.new_phase().await.unwrap();

        let _ = o.finalize().await;
        assert_eq!(counts.lock().unwrap().retrieve, 1);
    }

    #[tokio::test]
    async fn finalize_drains_handles() {
        let mut o = SourceOrchestrator::default();
        o.run(vec![mock_source()], &pid());
        let _ = o.finalize().await;
        assert!(o.handles.is_empty());
    }

    #[tokio::test]
    async fn measure_error_in_worker_propagates_to_orchestrator() {
        let mut r = MockMetricReader::new();
        r.expect_measure().returning(|| Err(MockError));
        let source: Box<dyn MetricSource> = r.into();
        let mut o = SourceOrchestrator::default();
        o.run(vec![source], &pid());

        o.measure().await.unwrap();
        let result = o.finalize().await;
        assert!(result.is_err());
    }
}
