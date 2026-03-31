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
    pub fn run(
        &mut self,
        sources: Vec<Box<dyn MetricSource>>,
        shared_pid: &Arc<AtomicI32>,
    ) -> Result<(), OrchestratorError> {
        if sources.is_empty() {
            return Err(OrchestratorError::NoSourceConfigured);
        }

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

        Ok(())
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
    struct State {
        pid: i32,
        init: usize,
        join: usize,
        measure: usize,
        reset: usize,
    }

    fn mock_reader() -> (MockMetricReader, Arc<Mutex<State>>) {
        let state_arc = Arc::new(Mutex::new(State::default()));
        let mut mock = MockMetricReader::new();

        let state = state_arc.clone();
        mock.expect_init().returning(move |pid| {
            let mut lock = state.lock().unwrap();
            lock.init += 1;
            lock.pid = pid;
            Ok(())
        });

        let state = state_arc.clone();
        mock.expect_join().returning(move || {
            state.lock().unwrap().join += 1;
            Ok(())
        });

        let state = state_arc.clone();
        mock.expect_measure().returning(move || {
            state.lock().unwrap().measure += 1;
            Ok(())
        });

        let state = state_arc.clone();
        mock.expect_reset().returning(move || {
            state.lock().unwrap().reset += 1;
            Ok(())
        });

        mock.expect_get_sensors().returning(|| Ok(vec![]));
        mock.expect_to_metrics()
            .returning(|_| Ok(Metrics::default()));

        (mock, state_arc)
    }

    fn pid() -> Arc<AtomicI32> {
        Arc::new(AtomicI32::new(0))
    }

    fn mock_source() -> (Box<dyn MetricSource>, Arc<Mutex<State>>) {
        let (r, state) = mock_reader();
        (r.into(), state)
    }

    #[tokio::test]
    async fn finalize_without_measurements_returns_not_enough_snapshots() {
        let mut orchestrator = SourceOrchestrator::default();
        let (source, _) = mock_source();
        orchestrator.run(vec![source], &pid()).unwrap();

        assert!(matches!(
            orchestrator.finalize().await,
            Err(OrchestratorError::NotEnoughSnapshots)
        ));
    }

    #[tokio::test]
    async fn run_orchestrator_with_no_source_returns_error() {
        let mut orchestrator = SourceOrchestrator::default();

        assert!(matches!(
            orchestrator.run(vec![], &pid()),
            Err(OrchestratorError::NoSourceConfigured)
        ));
    }

    #[tokio::test]
    async fn event_reaches_worker() {
        let (source, state) = mock_source();
        let mut orchestrator = SourceOrchestrator::default();
        orchestrator.run(vec![source], &pid()).unwrap();

        let _ = orchestrator.measure().await;
        let _ = orchestrator.reset().await;
        let _ = orchestrator.init().await;
        let _ = orchestrator.join().await;

        tokio::task::yield_now().await;

        let lock = state.lock().unwrap();

        assert_eq!(lock.measure, 1);
        assert_eq!(lock.init, 1);
        assert_eq!(lock.reset, 1);
        assert_eq!(lock.join, 1);
    }

    #[tokio::test]
    async fn init_initializes_source_with_right_pid() {
        let (source, state) = mock_source();

        let pid_value = 42;
        let shared_pid = pid();
        shared_pid.store(pid_value, Ordering::SeqCst);
        let mut orchestrator = SourceOrchestrator::default();
        orchestrator.run(vec![source], &shared_pid).unwrap();

        orchestrator.init().await.unwrap();

        let _ = orchestrator.finalize().await;
        assert_eq!(state.lock().unwrap().pid, pid_value);
    }

    #[tokio::test]
    async fn measure_error_in_worker_propagates_to_orchestrator() {
        let mut reader = MockMetricReader::new();
        reader.expect_measure().returning(|| Err(MockError));
        let source: Box<dyn MetricSource> = reader.into();
        let mut orchestrator = SourceOrchestrator::default();

        orchestrator.run(vec![source], &pid()).unwrap();
        orchestrator.measure().await.unwrap();
        let result = orchestrator.finalize().await;

        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(OrchestratorError::MetricSourceError(_))
        ));
    }
}
