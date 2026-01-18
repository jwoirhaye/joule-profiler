use std::{
    ops::{Add, AddAssign},
    pin::Pin,
    time::Duration,
};

use anyhow::Result;
use tokio::{select, sync::mpsc::Receiver, time::Instant};

use crate::core::{
    metric::{Metric, Metrics},
    sensor::Sensors,
};

#[derive(Default, Debug)]
pub struct SensorIteration {
    pub phases: Vec<SensorPhase>,
}

impl AddAssign for SensorPhase {
    fn add_assign(&mut self, rhs: Self) {
        self.metrics.extend(rhs.metrics);
    }
}

impl AddAssign for SensorIteration {
    fn add_assign(&mut self, rhs: Self) {
        self.phases
            .iter_mut()
            .zip(rhs.phases)
            .for_each(|(self_phase, rhs_phase)| *self_phase += rhs_phase);
    }
}

impl Add for SensorIteration {
    type Output = SensorIteration;

    fn add(mut self, rhs: Self) -> Self::Output {
        self += rhs;
        self
    }
}

#[derive(Default, Debug)]

pub struct SensorPhase {
    pub metrics: Vec<Metric>,
}

#[derive(Debug, Clone, Copy)]
pub enum SourceEvent {
    Measure,
    NewPhase,
    NewIteration,
    StartPolling,
    StopPolling,
    JoinWorker,
}

#[derive(Debug)]
pub struct SensorResult {
    pub iterations: Vec<SensorIteration>,
    pub count: u64,
    pub measure_delta: u64,
}

impl Add for SensorResult {
    type Output = SensorResult;

    fn add(self, rhs: Self) -> Self::Output {
        let count = self.count + rhs.count;
        let measure_delta = self.measure_delta + rhs.measure_delta;
        let iterations = self
            .iterations
            .into_iter()
            .zip(rhs.iterations)
            .map(|(self_iter, rhs_iter)| self_iter + rhs_iter)
            .collect();
        Self::Output {
            iterations,
            count,
            measure_delta,
        }
    }
}

impl SensorResult {
    pub fn merge(results: Vec<Self>) -> Option<SensorResult> {
        results.into_iter().reduce(|acc, result| acc + result)
    }
}
pub trait MetricReader {
    type Type: Into<Metrics> + AddAssign<Self::Type> + Default + Clone + PartialEq;

    /// Measure the sensors metrics.
    fn measure(&self) -> Result<Self::Type>;

    fn compute_measures(&self, new: &Self::Type, old: Self::Type) -> Result<Self::Type>;

    fn poll(&mut self) -> impl Future<Output = Option<Result<()>>> + Send + '_;
}

pub trait GetSensorsTrait: Send {
    fn get_sensors(&self) -> Result<Sensors>;
}

#[derive(Default, Clone)]
struct SourceIteration<V> {
    pub phases: Vec<SourcePhase<V>>,
}

#[derive(Default, Clone)]

struct SourcePhase<V> {
    pub metrics: V,
}

impl<V> SourcePhase<V> {
    pub fn new(metrics: V) -> Self {
        Self { metrics }
    }
}

impl<V> From<SourcePhase<V>> for SensorPhase
where
    V: Into<Metrics>,
{
    fn from(phase: SourcePhase<V>) -> Self {
        SensorPhase {
            metrics: phase.metrics.into(),
        }
    }
}

impl<V: Into<Metrics>> From<SourceIteration<V>> for SensorIteration {
    fn from(iteration: SourceIteration<V>) -> Self {
        let phases = iteration
            .phases
            .into_iter()
            .map(|phase| phase.into())
            .collect();
        SensorIteration { phases }
    }
}

#[derive(Clone)]
pub struct MetricSource<T: MetricReader + GetSensorsTrait> {
    metric_reader: T,

    iterations: Vec<SourceIteration<T::Type>>,

    current_iteration: SourceIteration<T::Type>,

    last_measure: Option<T::Type>,

    current_counters: T::Type,

    /// Number of snapshots taken
    count: u64,

    /// Total elapsed time between snapshots
    total_elapsed: Duration,

    /// Monotonic timestamp of last snapshot
    last_instant: Option<Instant>,

    polling_active: bool,
}

impl<T> MetricSource<T>
where
    T: MetricReader + GetSensorsTrait,
{
    pub fn new(reader: T) -> Self {
        Self {
            metric_reader: reader,
            iterations: Vec::new(),
            current_iteration: SourceIteration::default(),
            current_counters: T::Type::default(),
            last_measure: None,
            count: 0,
            total_elapsed: Duration::ZERO,
            last_instant: None,
            polling_active: false,
        }
    }

    /// Measure the sensors metrics.
    pub fn measure(&mut self) -> Result<()> {
        let now = Instant::now();
        if let Some(last) = self.last_instant {
            self.total_elapsed += now.duration_since(last);
        }

        self.last_instant = Some(now);
        self.count += 1;
        let measure = self.metric_reader.measure()?;

        if let Some(old) = self.last_measure.take() {
            self.current_counters += self.metric_reader.compute_measures(&measure, old)?;
        }

        self.last_measure = Some(measure);

        Ok(())
    }

    /// Initialize a new measure phase.
    pub fn new_phase(&mut self) -> Result<()> {
        if self.current_counters != T::Type::default() {
            let phase_counters = std::mem::take(&mut self.current_counters);
            self.current_iteration
                .phases
                .push(SourcePhase::new(phase_counters));
        }
        Ok(())
    }

    /// Initialize a new iteration.
    pub fn new_iteration(&mut self) -> Result<()> {
        if let Some(_) = self.last_measure.take() {
            let iteration = std::mem::take(&mut self.current_iteration);
            self.iterations.push(iteration);
        }
        Ok(())
    }

    /// Retrieve all sensors measures.
    pub fn retrieve(&mut self) -> Result<SensorResult> {
        let avg_delta_us = if self.count > 1 {
            (self.total_elapsed.as_micros() / (self.count - 1) as u128) as u64
        } else {
            0
        };

        let source_iterations = std::mem::take(&mut self.iterations);
        let iterations = source_iterations
            .into_iter()
            .map(|iteration| iteration.into())
            .collect();

        Ok(SensorResult {
            count: self.count,
            measure_delta: avg_delta_us,
            iterations,
        })
    }

    /// Start a worker thread to measure the source.
    pub async fn run_worker(&mut self, mut rx: Receiver<SourceEvent>) -> Result<SensorResult> {
        loop {
            select! {
                Some(event) = rx.recv() => {
                    match event {
                        SourceEvent::Measure => self.measure()?,
                        SourceEvent::NewPhase => self.new_phase()?,
                        SourceEvent::NewIteration => self.new_iteration()?,
                        SourceEvent::StartPolling => self.polling_active = true,
                        SourceEvent::StopPolling => self.polling_active = false,
                        SourceEvent::JoinWorker => return self.retrieve(),
                    }
                },
                Some(poll) = self.metric_reader.poll(), if self.polling_active => {
                    poll?;
                    self.measure()?;
                }
            }
        }
    }

    pub fn get_sensors(&self) -> Result<Sensors> {
        self.metric_reader.get_sensors()
    }
}

pub trait MetricSourceWorker: Send {
    fn run(
        self: Box<Self>,
        rx: Receiver<SourceEvent>,
    ) -> Pin<Box<dyn Future<Output = Result<SensorResult>> + Send>>;

    fn list_sensors(&self) -> Result<Sensors>;

    fn clone_box(&self) -> Box<dyn MetricSourceWorker>;
}

impl<T> From<T> for Box<dyn MetricSourceWorker>
where
    T: MetricReader + GetSensorsTrait + Send + 'static,
    MetricSource<T>: Clone,
    T::Type: Send,
{
    fn from(reader: T) -> Self {
        let source = MetricSource::new(reader);
        Box::new(source)
    }
}

impl<T> MetricSourceWorker for MetricSource<T>
where
    MetricSource<T>: Clone,
    T: MetricReader + GetSensorsTrait + Send + 'static,
    T::Type: Send,
{
    #[inline]
    fn run(
        mut self: Box<Self>,
        rx: Receiver<SourceEvent>,
    ) -> Pin<Box<dyn Future<Output = Result<SensorResult>> + Send>> {
        Box::pin(async move { self.run_worker(rx).await })
    }

    #[inline]
    fn list_sensors(&self) -> Result<Sensors> {
        self.get_sensors()
    }

    #[inline]
    fn clone_box(&self) -> Box<dyn MetricSourceWorker> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn MetricSourceWorker> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
