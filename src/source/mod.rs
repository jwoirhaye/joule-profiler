use std::{
    ffi::c_void,
    os::fd::RawFd,
    thread::{self, JoinHandle},
    time::Duration,
};

use anyhow::{Result, bail};
use crossbeam_channel::{Receiver, Sender};
use enum_dispatch::enum_dispatch;
use libc::{
    CLOCK_MONOTONIC, EPOLL_CLOEXEC, EPOLL_CTL_ADD, EPOLLIN, TFD_CLOEXEC, TFD_NONBLOCK,
    epoll_create1, epoll_ctl, epoll_event, epoll_wait, itimerspec, pipe, timerfd_create,
    timerfd_settime, timespec,
};
use log::{error, info};
use serde::Serialize;

use crate::source::rapl::Rapl;

pub mod rapl;

#[derive(Serialize, Clone, Debug)]
pub struct Metric {
    pub name: String,
    pub value: u64,
    pub unit: String,
    pub source: String,
}

pub type Metrics = Vec<Metric>;

#[derive(Debug, Clone, Copy)]
pub enum SourceEvent {
    Measure,
    Phase,
    Start,
    Pause,
    Stop,
}

#[enum_dispatch]
pub trait MetricReader {
    /// Measure the sensors metrics.
    fn measure(&mut self) -> Result<()>;

    /// Initialize a new measure phase.
    fn phase(&mut self) -> Result<()>;

    /// Retrieve all sensors measures.
    fn retrieve(&mut self) -> Result<SourceResult>;

    /// Get all the metric source sensors.
    fn get_sensors(&self) -> Result<Vec<Sensor>>;

    /// Get the polling interval of the metric source if supported.
    fn get_polling_interval(&self) -> Option<Duration> {
        None
    }

    fn get_name(&self) -> &'static str;
}

#[enum_dispatch(MetricReader)]
#[derive(Clone, Debug)]
pub enum MetricSource {
    Rapl(Rapl),
}

pub struct SourceResult {
    pub measures: Vec<Metrics>,
    pub count: u64,
}

pub struct SourceManager {
    sources: Vec<MetricSource>,
    senders: Vec<Sender<SourceEvent>>,
    handles: Vec<JoinHandle<Result<SourceResult>>>,
}

impl SourceManager {
    pub fn new(sources: Vec<MetricSource>) -> Self {
        Self {
            sources,
            senders: Vec::new(),
            handles: Vec::new(),
        }
    }

    /// Start the metrics sources worker threads.
    pub fn start_workers(&mut self) {
        let sources = self.sources.clone();
        let mut senders = Vec::new();
        let mut handles = Vec::new();

        for source in sources {
            let (tx, rx) = crossbeam_channel::bounded(4);
            senders.push(tx.clone());

            let handle = thread::spawn(move || {
                let poll_interval = source.get_polling_interval();

                info!("Worker started for source {:?}", source.get_name());

                match poll_interval {
                    Some(interval) => run_worker_with_polling(source, rx, interval),
                    None => run_worker_event_only(source, rx),
                }
            });

            handles.push(handle);
        }

        self.handles = handles;
        self.senders = senders;
    }

    /// Send an event to each metrics source.
    pub fn send_event(&self, event: SourceEvent) -> Result<()> {
        for sender in &self.senders {
            sender.send(event)?;
        }
        Ok(())
    }

    /// Start the polling of a metrics source if enabled.
    pub fn start(&self) -> Result<()> {
        self.send_event(SourceEvent::Start)
    }

    /// Measure the metrics of each metrics source.
    pub fn measure(&self) -> Result<()> {
        self.send_event(SourceEvent::Measure)
    }

    /// Initialize a new phase for each metrics source.
    pub fn phase(&self) -> Result<()> {
        self.send_event(SourceEvent::Phase)
    }

    /// Pause the polling of a metrics source if enabled.
    pub fn pause(&self) -> Result<()> {
        self.send_event(SourceEvent::Pause)
    }

    /// Stop the worker thread of each metrics sources to join threads gracefully.
    pub fn stop(&self) -> Result<()> {
        self.send_event(SourceEvent::Stop)
    }

    /// Gracefully shutdown all the workers.
    pub fn join(&mut self) -> Result<SourceResult> {
        info!("Stopping all workers");
        self.stop()?;

        let handles = std::mem::take(&mut self.handles);
        let mut all_phases = Vec::new();

        for handle in handles {
            match handle.join() {
                Ok(Ok(phases)) => all_phases.push(phases),
                Ok(Err(e)) => error!("Worker returned error: {:?}", e),
                Err(_) => error!("Worker panicked"),
            }
        }

        info!("All workers joined. Merging phases");

        let max_phases = all_phases
            .iter()
            .map(|source_result| source_result.measures.len())
            .max()
            .unwrap_or(0);
        let mut merged = Vec::with_capacity(max_phases);

        let mut measure_count = 0;

        for i in 0..max_phases {
            let mut phase_metrics = Vec::new();
            for source_result in &all_phases {
                measure_count += source_result.count;
                if let Some(measures) = source_result.measures.get(i) {
                    phase_metrics.extend(measures.clone());
                }
            }
            merged.push(phase_metrics);
        }

        info!("Merged {} phases", merged.len());

        Ok(SourceResult {
            measures: merged,
            count: measure_count,
        })
    }
}

#[derive(Serialize)]
pub struct Sensor {
    pub name: String,
    pub unit: String,
    pub source: String,
}

/// Start a worker without polling.
fn run_worker_event_only<S: MetricReader>(
    mut source: S,
    rx: Receiver<SourceEvent>,
) -> Result<SourceResult> {
    loop {
        match rx.recv() {
            Ok(SourceEvent::Stop) => return source.retrieve(),
            Ok(event) => handle_event(&mut source, event),
            Err(_) => return source.retrieve(),
        }
    }
}

/// Handle an event for a no-polling worker (only phase and measure events supported).
fn handle_event<S: MetricReader>(source: &mut S, event: SourceEvent) {
    match event {
        SourceEvent::Phase => {
            if let Err(e) = source.phase() {
                error!("Phase error: {:?}", e);
            }
        }
        SourceEvent::Measure => {
            if let Err(e) = source.measure() {
                error!("Measure error: {:?}", e);
            }
        }
        _ => {}
    }
}

/// Run a worker with polling enabled, uses timerfd and epoll for maximum performance.
fn run_worker_with_polling<S: MetricReader>(
    mut source: S,
    rx: Receiver<SourceEvent>,
    interval: Duration,
) -> Result<SourceResult> {
    let timer_fd = unsafe { timerfd_create(CLOCK_MONOTONIC, TFD_NONBLOCK | TFD_CLOEXEC) };

    if timer_fd < 0 {
        bail!("timerfd_create failed");
    }

    let timer_interval = itimerspec {
        it_interval: timespec {
            tv_sec: interval.as_secs() as i64,
            tv_nsec: interval.subsec_nanos() as i64,
        },
        it_value: timespec {
            tv_sec: interval.as_secs() as i64,
            tv_nsec: interval.subsec_nanos() as i64,
        },
    };

    if unsafe { timerfd_settime(timer_fd, 0, &timer_interval, std::ptr::null_mut()) } != 0 {
        anyhow::bail!("timerfd_settime failed");
    }

    let mut pipe_fds = [0; 2];
    unsafe {
        if pipe(pipe_fds.as_mut_ptr()) != 0 {
            anyhow::bail!("pipe failed");
        }
    }
    let pipe_r = pipe_fds[0];
    let pipe_w = pipe_fds[1];

    let rx_clone = rx.clone();
    thread::spawn(move || {
        while let Ok(event) = rx_clone.recv() {
            let byte = match event {
                SourceEvent::Start => 1u8,
                SourceEvent::Pause => 2u8,
                SourceEvent::Stop => 3u8,
                SourceEvent::Measure => 4u8,
                SourceEvent::Phase => 5u8,
            };
            let _ = unsafe { libc::write(pipe_w, &byte as *const u8 as *const _, 1) };
        }
    });

    let epfd = unsafe {
        let fd = epoll_create1(EPOLL_CLOEXEC);
        if fd < 0 {
            anyhow::bail!("epoll_create1 failed");
        }
        fd
    };

    let mut ev = epoll_event {
        events: EPOLLIN as u32,
        u64: timer_fd as u64,
    };
    if unsafe { epoll_ctl(epfd, EPOLL_CTL_ADD, timer_fd, &mut ev) != 0 } {
        anyhow::bail!("epoll_ctl add timer_fd failed");
    }

    let mut ev_pipe = epoll_event {
        events: EPOLLIN as u32,
        u64: pipe_r as u64,
    };
    if unsafe { epoll_ctl(epfd, EPOLL_CTL_ADD, pipe_r, &mut ev_pipe) != 0 } {
        anyhow::bail!("epoll_ctl add pipe_r failed");
    }

    let mut polling_active = true;
    let mut events = [epoll_event { events: 0, u64: 0 }; 10];

    loop {
        let nfds = unsafe { epoll_wait(epfd, events.as_mut_ptr(), events.len() as i32, -1) };
        if nfds < 0 {
            anyhow::bail!("epoll_wait failed");
        }

        for i in 0..nfds as usize {
            let fd = events[i].u64 as RawFd;

            if fd == timer_fd && polling_active {
                source.measure()?;
                let mut buf: u64 = 0;
                unsafe { libc::read(timer_fd, &mut buf as *mut u64 as *mut c_void, 8) };
            }

            if fd == pipe_r {
                let mut byte = [0u8; 1];
                let n = unsafe { libc::read(pipe_r, byte.as_mut_ptr() as *mut _, 1) };
                if n > 0 {
                    match byte[0] {
                        1 => polling_active = true,
                        2 => polling_active = false,
                        3 => return source.retrieve(),
                        4 => {
                            source.measure()?;
                        }
                        5 => {
                            source.phase()?;
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}
