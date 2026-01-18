use std::{
    ops::{Add, AddAssign},
    pin::Pin,
    time::Duration,
};

use anyhow::Result;
use tokio::{select, sync::mpsc::Receiver, time::Instant};

use crate::core::{
    metric::{Metric, Metrics},
    sensor::{SensorResult, Sensors},
};

#[derive(Default, Debug)]
pub struct SensorIteration {
    pub phases: Vec<SensorPhase>,
    pub measure_delta: u64,
    pub measure_count: u64,
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

impl SensorIteration {
    pub fn new(phases: Vec<SensorPhase>, measure_delta: u64, measure_count: u64) -> Self {
        Self {
            phases,
            measure_delta,
            measure_count,
        }
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

pub trait MetricReaderBound: MetricReader + GetSensorsTrait + Send + 'static {}
impl<T> MetricReaderBound for T where T: Clone + GetSensorsTrait + MetricReader + Send + 'static {}

pub trait MetricReaderTypeBound<T>: Into<Metrics> + AddAssign<T> + Default + Clone + PartialEq {}
impl<T> MetricReaderTypeBound<T> for T where T: Into<Metrics> + AddAssign<T> + Default + Clone + PartialEq {}

pub trait MetricReader {
    type Type: MetricReaderTypeBound<Self::Type>;

    fn init(&mut self) -> Result<()>;

    /// Measure the sensors metrics.
    fn measure(&self) -> Result<Self::Type>;

    fn compute_measures(&self, new: &Self::Type, old: Self::Type) -> Result<Self::Type>;

    fn poll(&mut self) -> impl Future<Output = Option<Result<()>>> + Send;
}

pub trait GetSensorsTrait {
    fn get_sensors(&self) -> Result<Sensors>;
}

#[derive(Default, Clone)]
struct SourceIteration<V> {
    pub phases: Vec<SourcePhase<V>>,
    pub total_elapsed: Duration,
    pub measure_count: u64,
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

        let measure_delta = if iteration.measure_count > 1 {
            (iteration.total_elapsed.as_micros() / (iteration.measure_count - 1) as u128) as u64
        } else {
            0
        };

        SensorIteration::new(phases, measure_delta, iteration.measure_count)
    }
}

#[derive(Clone)]
pub struct MetricSource<T: MetricReaderBound> {
    metric_reader: T,

    iterations: Vec<SourceIteration<T::Type>>,

    current_iteration: SourceIteration<T::Type>,

    last_measure: Option<T::Type>,

    current_counters: T::Type,

    /// Monotonic timestamp of last snapshot
    last_instant: Option<Instant>,

    polling_active: bool,
}

impl<T: MetricReaderBound> MetricSource<T>
{
    pub fn new(reader: T) -> Self {
        Self {
            metric_reader: reader,
            iterations: Vec::new(),
            current_iteration: SourceIteration::default(),
            current_counters: T::Type::default(),
            last_measure: None,
            last_instant: None,
            polling_active: false,
        }
    }

    /// Measure the sensors metrics.
    pub fn measure(&mut self) -> Result<()> {
        let now = Instant::now();
        if let Some(last) = self.last_instant {
            self.current_iteration.total_elapsed += now.duration_since(last);
        }

        self.last_instant = Some(now);
        self.current_iteration.measure_count += 1;
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
        if self.last_measure.take().is_some() {
            let iteration = std::mem::take(&mut self.current_iteration);
            self.current_iteration.measure_count = 0;
            self.current_iteration.total_elapsed = Duration::ZERO;
            self.last_instant = None;
            self.last_measure = None;
            self.iterations.push(iteration);
        }
        Ok(())
    }

    /// Retrieve all sensors measures.
    pub fn retrieve(&mut self) -> Result<SensorResult> {
        let source_iterations = std::mem::take(&mut self.iterations);
        let iterations = source_iterations
            .into_iter()
            .map(|iteration| iteration.into())
            .collect();
        let result = SensorResult::new(iterations);
        Ok(result)
    }

    /// Start a worker thread to measure the source.
    pub async fn run_worker(&mut self, mut rx: Receiver<SourceEvent>) -> Result<SensorResult> {
        self.metric_reader.init()?;
        loop {
            select! {
                Some(poll) = self.metric_reader.poll(), if self.polling_active => {
                    poll?;
                    self.measure()?;
                }
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
    MetricSource<T>: Clone,
    T: MetricReaderBound,
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
    T: MetricReaderBound,
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
