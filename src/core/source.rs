use std::{marker::PhantomData, ops::{AddAssign}, time::Duration};

use anyhow::Result;
use enum_dispatch::enum_dispatch;
use tokio::{
    select,
    sync::mpsc::Receiver,
    time::{Instant, MissedTickBehavior, interval},
};

use crate::core::{
    metric::{Metric, Metrics},
    sensor::Sensor,
};

#[derive(Default)]
struct Iteration {
    pub phases: Vec<Phase>,
}

#[derive(Default)]

struct Phase {
    pub metrics: Vec<Metric>,
}

#[derive(Debug, Clone, Copy)]
pub enum SourceEvent {
    Measure,
    Phase,
    Start,
    Stop,
    Join,
}

pub struct SourceResult {
    pub iterations: Vec<Iteration>,
    pub count: u64,
    pub measure_delta: u128,
}

#[enum_dispatch]
pub trait MetricReader<V> {
    /// Measure the sensors metrics.
    fn measure(&mut self) -> Result<V>;

    /// Get all the metric source sensors.
    fn get_sensors(&self) -> Result<Vec<Sensor>>;

    /// Get the polling interval of the metric source if supported.
    fn get_polling_interval(&self) -> Option<Duration> {
        None
    }

    fn compute_measures(&self, new: V, old: V) -> Result<V>;
}

#[derive(Default)]
struct SourceIteration<V> {
    pub phases: Vec<SourcePhase<V>>,
}

#[derive(Default)]

struct SourcePhase<V> {
    pub metrics: V,
}

impl<V> From<SourcePhase<V>> for Phase where V: Into<Metrics> {
    fn from(phase: SourcePhase<V>) -> Self {
        Phase { metrics: phase.metrics.into() }
    }
}

impl<V: Into<Metrics>> From<SourceIteration<V>> for Iteration {
    fn from(iteration: SourceIteration<V>) -> Self {
        let phases = iteration.phases.into_iter().map(|phase| phase.into()).collect();
        Iteration { phases }
    }
}

pub struct MetricSource<V, T: MetricReader<V>> {
    metric_reader: T,

    _result_type: PhantomData<V>,

    iterations: Vec<SourceIteration<V>>,

    current_iteration: SourceIteration<V>,

    last_measure: Option<V>,

    current_counter: V,

    /// Number of snapshots taken
    count: u64,

    /// Total elapsed time between snapshots
    total_elapsed: Duration,

    /// Monotonic timestamp of last snapshot
    last_instant: Option<Instant>,
}

impl<V, T> MetricSource<V, T>
where
    T: MetricReader<V>,
    V: Into<Metrics> + AddAssign<V> + Default,
{
    pub fn new(reader: T) -> Self {
        Self {
            metric_reader: reader,
            _result_type: PhantomData,
            iterations: Vec::new(),
            current_iteration: SourceIteration::default(),
            current_counter: V::default(),
            last_measure: None,
            count: 0,
            total_elapsed: Duration::ZERO,
            last_instant: None,
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
            let diff = self.metric_reader.compute_measures(measure, old)?;
            self.current_counter += diff;
        }

        Ok(())
    }

    /// Initialize a new measure phase.
    pub fn phase(&mut self) -> Result<()> {
        self.measure()?;
        let phase_counters = std::mem::take(&mut self.current_counter);
        // let iteration = std::mem::take(&mut self.current_iteration);
        self.current_iteration.phases.push(SourcePhase { metrics: phase_counters });
        Ok(())
    }

    pub fn iteration(&mut self) -> Result<()> {
        self.phase()?;
        let iteration = std::mem::take(&mut self.current_iteration);
        self.iterations.push(iteration);
        Ok(())
    }

    /// Retrieve all sensors measures.
    pub fn retrieve(&mut self) -> Result<SourceResult> {
        let avg_delta_us = if self.count > 1 {
            self.total_elapsed.as_micros() / (self.count - 1) as u128
        } else {
            0
        };
        
        let source_iterations = std::mem::take(&mut self.iterations);
        let iterations = source_iterations.into_iter().map(|iteration| iteration.into()).collect();

        Ok(SourceResult {
            count: self.count,
            measure_delta: avg_delta_us,
            iterations,
        })
    }

    /// Start a worker thread to measure the source.
    pub async fn run_worker(&mut self, rx: Receiver<SourceEvent>) -> Result<SourceResult> {
        match self.metric_reader.get_polling_interval() {
            Some(polling_interval) => self.run_worker_with_polling(rx, polling_interval).await,
            None => self.run_worker_event_only(rx).await,
        }
    }

    /// Start a worker without polling.
    pub async fn run_worker_event_only(
        &mut self,
        mut rx: Receiver<SourceEvent>,
    ) -> Result<SourceResult> {
        loop {
            match rx.recv().await {
                Some(SourceEvent::Stop) => return self.retrieve(),
                Some(event) => self.handle_event_no_polling(event)?,
                _ => {}
            }
        }
    }

    /// Start a worker with polling.
    pub async fn run_worker_with_polling(
        &mut self,
        mut rx: Receiver<SourceEvent>,
        polling_interval: Duration,
    ) -> Result<SourceResult> {
        let mut polling_active = true;

        let mut reload_timer = interval(polling_interval);
        reload_timer.set_missed_tick_behavior(MissedTickBehavior::Delay);

        loop {
            select! {
                Some(event) = rx.recv() => {
                    match event {
                        SourceEvent::Start => polling_active = true,
                        SourceEvent::Stop => polling_active = false,
                        SourceEvent::Join => return self.retrieve(),
                        SourceEvent::Measure => {
                            self.measure()?;
                        },
                        SourceEvent::Phase => {
                            self.phase()?;
                        },
                    }
                }
                _ = reload_timer.tick() => {
                    if polling_active {
                        self.measure()?;
                    }
                }
            }
        }
    }

    /// Handle an event for a no-polling worker (only phase and measure events supported).
    fn handle_event_no_polling(&mut self, event: SourceEvent) -> Result<()> {
        match event {
            SourceEvent::Phase => {
                self.phase()?;
            }
            SourceEvent::Measure => {
                self.measure()?;
            }
            _ => {}
        }
        Ok(())
    }
}
