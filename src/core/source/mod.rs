use std::{
    fmt::Debug,
    ops::{Add, AddAssign},
    pin::Pin,
    time::Duration,
};

use tokio::{select, sync::mpsc::Receiver, time::Instant};

use crate::core::{
    metric::{Metric, Metrics},
    phase::SourcePhase,
    sensor::Sensors,
    source::error::MetricSourceError,
};

pub mod error;

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

#[derive(Debug)]
pub struct SensorResult {
    pub iterations: Vec<SensorIteration>,
}

impl Add for SensorResult {
    type Output = SensorResult;

    fn add(self, rhs: Self) -> Self::Output {
        let iterations = self
            .iterations
            .into_iter()
            .zip(rhs.iterations)
            .map(|(self_iter, rhs_iter)| self_iter + rhs_iter)
            .collect();
        Self::Output { iterations }
    }
}

impl SensorResult {
    pub fn new(iterations: Vec<SensorIteration>) -> Self {
        Self { iterations }
    }

    pub fn merge(results: Vec<Self>) -> Option<SensorResult> {
        results.into_iter().reduce(|acc, result| acc + result)
    }
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

pub trait MetricReaderTypeBound:
    Debug + Default + Clone + PartialEq + Send + Into<Metrics> + AddAssign<Self>
{
}

impl<T> MetricReaderTypeBound for T where
    T: Debug + Default + Clone + PartialEq + Send + Into<Metrics> + AddAssign<Self>
{
}

pub trait MetricReaderTrait: Send + 'static {
    type Type: MetricReaderTypeBound;
    type Error: Debug + Into<MetricSourceError>;

    /// Measure the sensors metrics.
    fn measure(&self) -> Result<Self::Type, Self::Error>;

    fn compute_measures(
        &self,
        new: &Self::Type,
        old: Self::Type,
    ) -> Result<Self::Type, Self::Error>;

    fn poll(&mut self) -> impl Future<Output = Option<Result<(), Self::Error>>> + Send;

    fn get_sensors(&self) -> Result<Sensors, Self::Error>;
}

#[derive(Debug, Default, Clone)]
struct SourceIteration<V> {
    pub phases: Vec<SourcePhase<V>>,
    pub total_elapsed: Duration,
    pub measure_count: u64,
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

#[derive(Debug)]
pub struct MetricReader<R: MetricReaderTrait> {
    metric_reader: R,

    iterations: Vec<SourceIteration<R::Type>>,

    current_iteration: SourceIteration<R::Type>,

    last_measure: Option<R::Type>,

    current_counters: R::Type,

    /// Monotonic timestamp of last snapshot
    last_instant: Option<Instant>,

    polling_active: bool,
}

impl<T: MetricReaderTrait> MetricReader<T> {
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
    pub fn measure(&mut self) -> Result<(), MetricSourceError> {
        let now = Instant::now();
        if let Some(last) = self.last_instant {
            self.current_iteration.total_elapsed += now.duration_since(last);
        }

        self.last_instant = Some(now);
        self.current_iteration.measure_count += 1;
        let measure = self.metric_reader.measure().map_err(|err| err.into())?;

        if let Some(old) = self.last_measure.take() {
            self.current_counters += self
                .metric_reader
                .compute_measures(&measure, old)
                .map_err(|err| err.into())?;
        }

        self.last_measure = Some(measure);

        Ok(())
    }

    /// Initialize a new measure phase.
    pub fn new_phase(&mut self) -> Result<(), MetricSourceError> {
        if self.current_counters != T::Type::default() {
            let phase_counters = std::mem::take(&mut self.current_counters);
            self.current_iteration
                .phases
                .push(SourcePhase::new(phase_counters));
        }
        Ok(())
    }

    /// Initialize a new iteration.
    pub fn new_iteration(&mut self) -> Result<(), MetricSourceError> {
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
    pub fn retrieve(self) -> Result<(SensorResult, Box<dyn MetricSource>), MetricSourceError> {
        let iterations = self
            .iterations
            .into_iter()
            .map(|iteration| iteration.into())
            .collect();
        let result = SensorResult::new(iterations);
        let boxed_source = Box::new(MetricReader::new(self.metric_reader));
        Ok((result, boxed_source))
    }

    /// Start a worker thread to measure the source.
    pub async fn run_worker(
        mut self,
        mut rx: Receiver<SourceEvent>,
    ) -> Result<(SensorResult, Box<dyn MetricSource>), MetricSourceError> {
        loop {
            select! {
                Some(poll) = self.metric_reader.poll(), if self.polling_active => {
                    poll.map_err(|err| err.into())?;
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

    pub fn get_sensors(&self) -> Result<Sensors, MetricSourceError> {
        self.metric_reader.get_sensors().map_err(|err| err.into())
    }
}

type MetricSourceWorkerFuture = Pin<
    Box<
        dyn Future<Output = Result<(SensorResult, Box<dyn MetricSource>), MetricSourceError>>
            + Send,
    >,
>;

pub trait MetricSource: Send {
    /// Runs the worker and returns the result along with the source itself.
    fn run(self: Box<Self>, rx: Receiver<SourceEvent>) -> MetricSourceWorkerFuture;

    fn list_sensors(&self) -> Result<Sensors, MetricSourceError>;

    fn into_box(self) -> Box<dyn MetricSource>
    where
        Self: Sized + 'static,
    {
        Box::new(self)
    }
}

impl<T> MetricSource for MetricReader<T>
where
    T: MetricReaderTrait,
{
    fn run(self: Box<Self>, rx: Receiver<SourceEvent>) -> MetricSourceWorkerFuture {
        Box::pin(async move { self.run_worker(rx).await })
    }

    fn list_sensors(&self) -> Result<Sensors, MetricSourceError> {
        self.get_sensors()
    }
}

impl<T> From<T> for Box<dyn MetricSource>
where
    T: MetricReaderTrait,
{
    fn from(reader: T) -> Self {
        let source = MetricReader::new(reader);
        Box::new(source)
    }
}
