use std::{
    io::Error,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};

use futures::StreamExt;
use joule_profiler_core::{
    sensor::{Sensor, Sensors},
    source::MetricReader,
    types::{Metric, Metrics},
    unit::{MetricUnit, Unit, UnitPrefix},
};
use procfs::process::Process;
use thiserror::Error;
use tokio::task::JoinHandle;
use tokio_timerfd::Interval;

type Result<T> = std::result::Result<T, ProcfsError>;

#[derive(Debug, Error)]
pub enum ProcfsError {
    #[error("Unable to create procfs process, {0}")]
    ProcessCreationError(#[from] procfs::ProcError),

    #[error("Not enough samples to compute a phase")]
    NotEnoughSamples,

    #[error("I/O error: {0}")]
    IoError(#[from] Error),
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Snapshot {
    pub rss_kb: u64,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Phase {
    pub begin: Snapshot,
    pub end: Snapshot,
    pub max: u64,
}

pub struct Procfs {
    process: Process,

    begin_snapshot: Option<Snapshot>,
    end_snapshot: Option<Snapshot>,

    phase_max: Arc<AtomicU64>,

    polling_interval: Option<Duration>,

    handle: Option<JoinHandle<Result<()>>>,
}

const KILO_BYTE_UNIT: MetricUnit = MetricUnit {
    prefix: UnitPrefix::Kilo,
    unit: Unit::Byte,
};

impl Procfs {
    pub fn new(polling_interval: Option<Duration>) -> Result<Self> {
        Ok(Self {
            process: Process::myself()?,
            begin_snapshot: None,
            end_snapshot: None,
            phase_max: Arc::new(AtomicU64::new(0)),
            polling_interval,
            handle: None,
        })
    }

    fn read_memory(&self) -> Result<Option<Snapshot>> {
        if !self.process.is_alive() {
            return Ok(None);
        }

        let status = self.process.status()?;

        let vmrss_kb = status.vmrss.ok_or(ProcfsError::NotEnoughSamples)?;

        Ok(Some(Snapshot { rss_kb: vmrss_kb }))
    }

    fn start_polling(&mut self, pid: i32) -> Result<()> {
        let Some(interval) = self.polling_interval else {
            return Ok(());
        };

        let process = Process::new(pid)?;
        let max = self.phase_max.clone();

        let handle = tokio::spawn(async move {
            let mut ticker = Interval::new_interval(interval)?;

            loop {
                ticker.next().await;

                if !process.is_alive() {
                    continue;
                }

                if let Ok(status) = process.status() {
                    if let Some(vmrss_kb) = status.vmrss {
                        let mut current = max.load(Ordering::Relaxed);

                        while vmrss_kb > current {
                            match max.compare_exchange(
                                current,
                                vmrss_kb,
                                Ordering::Relaxed,
                                Ordering::Relaxed,
                            ) {
                                Ok(_) => break,
                                Err(v) => current = v,
                            }
                        }
                    }
                }
            }
        });

        self.handle = Some(handle);
        Ok(())
    }
}

impl MetricReader for Procfs {
    type Type = Phase;
    type Error = ProcfsError;

    async fn init(&mut self, pid: i32) -> Result<()> {
        self.process = Process::new(pid)?;

        let snap = self.read_memory()?;

        if let Some(snapshot) = snap {
            self.phase_max.store(snapshot.rss_kb, Ordering::Relaxed);
            self.begin_snapshot = Some(snapshot);
            self.end_snapshot = Some(snapshot);
        }

        self.start_polling(pid)?;

        Ok(())
    }
    async fn measure(&mut self) -> Result<()> {
        let snap = if self.process.is_alive()
            && let Some(snap) = self.read_memory()?
        {
            snap
        } else {
            return Ok(());
        };

        self.end_snapshot = Some(snap);

        if self.begin_snapshot.is_none() {
            self.begin_snapshot = Some(snap);
            self.phase_max.store(snap.rss_kb, Ordering::Relaxed);
            return Ok(());
        }

        let mut current_max = self.phase_max.load(Ordering::Relaxed);

        if snap.rss_kb > current_max {
            while let Err(v) = self.phase_max.compare_exchange(
                current_max,
                snap.rss_kb,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                current_max = v;
                if snap.rss_kb <= current_max {
                    break;
                }
            }
        }

        Ok(())
    }

    async fn retrieve(&mut self) -> Result<Self::Type> {
        let begin = self
            .begin_snapshot
            .take()
            .ok_or(ProcfsError::NotEnoughSamples)?;

        let end = self.end_snapshot.unwrap_or(begin);

        let max = self.phase_max.load(Ordering::Relaxed);

        self.phase_max.store(0, Ordering::Relaxed);
        self.begin_snapshot = Some(end);
        self.end_snapshot = Some(end);

        Ok(Phase { begin, end, max })
    }

    fn get_sensors(&self) -> Result<Sensors> {
        Ok(vec![
            Sensor {
                name: "memory_rss_delta".into(),
                unit: KILO_BYTE_UNIT,
                source: Self::get_name().to_string(),
            },
            Sensor {
                name: "memory_rss_max".into(),
                unit: KILO_BYTE_UNIT,
                source: Self::get_name().to_string(),
            },
            Sensor {
                name: "memory_rss_end".into(),
                unit: KILO_BYTE_UNIT,
                source: Self::get_name().to_string(),
            },
        ]
        .into())
    }

    fn to_metrics(&self, phase: Self::Type) -> Result<Metrics> {
        let delta_kb: i64 = phase.end.rss_kb as i64 - phase.begin.rss_kb as i64;

        Ok(vec![
            Metric::new(
                "memory_rss_delta",
                delta_kb,
                KILO_BYTE_UNIT,
                Self::get_name(),
            ),
            Metric::new(
                "memory_rss_max",
                phase.max,
                KILO_BYTE_UNIT,
                Self::get_name(),
            ),
            Metric::new(
                "memory_rss_end",
                phase.end.rss_kb,
                KILO_BYTE_UNIT,
                Self::get_name(),
            ),
        ])
    }

    fn get_name() -> &'static str {
        "procfs"
    }
}
