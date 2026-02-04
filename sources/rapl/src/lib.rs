//! Module `rapl` — Intel RAPL metric source.
//!
//! This module provides an implementation of a [`MetricReader`] for
//! collecting energy metrics from Intel RAPL (Running Average Power Limit) domains.
//!
//! The `Rapl` struct manages RAPL domains, reads energy counters,
//! and optionally supports periodic polling for continuous measurement.
//!
//! # Features
//!
//! - Discover available RAPL domains under a given path.
//! - Read instantaneous energy consumption snapshots.
//! - Compute energy usage between consecutive snapshots.
//! - Provide sensors information for integration with the profiler.
//! - Optional async scheduler for periodic measurement.
//!
//! # Usage
//!
//! ```no_run
//! use source_rapl::Rapl;
//! use joule_profiler_core::source::MetricReader;
//!
//! #[tokio::main]
//! async fn main() {
//!     // Initialize a RAPL reader (no polling, monitoring all sockets)
//!     let mut rapl = Rapl::from_default().unwrap();
//!
//!     // Measure and update internal counters
//!     rapl.measure().await.unwrap();
//!
//!     // Retrieve available sensors
//!     let sensors = rapl.get_sensors().unwrap();
//!
//!     // Retrieve collected counters
//!     let counters = rapl.retrieve().await.unwrap();
//! }
//! ```
//!
//! # Errors
//!
//! All RAPL operations return a [`RaplError`]. Possible errors include:
//! - [`RaplError::RaplNotAvailable`] - no RAPL domains found at the specified path.
//! - [`RaplError::InsufficientPermissions`] - requires elevated privileges to read powercap files.
//! - [`RaplError::UnsupportedOS`] - only Linux is supported.
//! - [`RaplError::RaplReadError`] or [`RaplError::InvalidRaplPath`] - problems reading counters or invalid paths.

use crate::domain::{RaplDomain, get_domains, read_energy};
use crate::error::RaplError;
use crate::snapshot::{Snapshot, compute_measurement_from_snapshots};
use futures::StreamExt;
use joule_profiler_core::sensor::{Sensor, Sensors};
use joule_profiler_core::source::MetricReader;
use joule_profiler_core::types::Metrics;
use joule_profiler_core::unit::{MetricUnit, SIPrefix, Unit};
use log::{debug, error, info, trace};
use std::fs;
use std::io::ErrorKind;
use std::path::Path;
use std::sync::Arc;
use std::{
    collections::{HashMap, HashSet},
    env,
    time::Duration,
};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio_timerfd::Interval;
mod domain;
pub mod error;
mod snapshot;

const DEFAULT_RAPL_PATH: &str = "/sys/devices/virtual/powercap/intel-rapl";
const POWERCAP_SOURCE_NAME: &str = "Powercap";
const MICRO_JOULE_UNIT: MetricUnit = MetricUnit {
    prefix: SIPrefix::Micro,
    unit: Unit::Joule,
};

/// Custom result type for Rapl
type Result<T> = std::result::Result<T, RaplError>;

/// RAPL metric source
///
/// Implements [`MetricReader`] and provides energy metrics from Intel RAPL.
///
/// # Fields
///
/// - `domains`: Managed RAPL domains discovered under the base path. Each domain corresponds
///   to a CPU socket and an energy domain.
/// - `current_counters`: Latest energy counters collected by this reader. Updated by
///   [`Self::measure`] and returned by [`Self::retrieve`].
/// - `last_snapshot`: Last snapshot read from RAPL domains, used to compute the energy delta
///   between measurements.
pub struct Rapl {
    rapl_path: String,

    domains: Vec<RaplDomain>,

    poll_interval: Option<Duration>,

    current_counters: Arc<Mutex<Snapshot>>,

    last_snapshot: Arc<Mutex<Option<Snapshot>>>,

    handle: Option<JoinHandle<Result<()>>>,
}

impl Rapl {
    /// Creates a new RAPL reader for the given path and sockets.
    ///
    /// `rapl_path` — base path to RAPL domains (e.g., `/sys/devices/virtual/powercap/intel-rapl`)  
    /// `sockets` — optional set of CPU sockets to monitor  
    /// `polling_rate_s` — optional interval in seconds for periodic measurement
    ///
    /// # Errors
    ///
    /// Returns a `RaplError` if:
    /// - RAPL interface is unavailable
    /// - Path is invalid
    /// - Permissions are insufficient
    pub fn new(
        rapl_path: Option<&str>,
        sockets: Option<&str>,
        polling_rate_s: Option<f64>,
    ) -> Result<Self> {
        let rapl_path = rapl_base_path(rapl_path);

        trace!(
            "Attempting to initialize RAPL reader: rapl_path={}, sockets={:?}",
            rapl_path, sockets
        );

        let sockets: Option<HashSet<u32>> = sockets.map(|s| {
            s.split(',')
                .filter_map(|x| x.trim().parse::<u32>().ok())
                .collect()
        });

        check_os()?;

        let domains = get_domains(&rapl_path, sockets.as_ref())?;

        if domains.is_empty() {
            error!("No domain discovered");
            return Err(RaplError::NoDomains);
        }

        info!("Discovered {} RAPL domain(s)", domains.len());

        let poll_interval = polling_rate_s.map(Duration::from_secs_f64);

        trace!(
            "Creating Rapl instance (domains={}, ticker={})",
            domains.len(),
            poll_interval.is_some()
        );

        Ok(Rapl {
            rapl_path,
            domains,
            poll_interval,
            current_counters: Default::default(),
            last_snapshot: Default::default(),
            handle: None,
        })
    }

    /// Initializes a Rapl source with default configuration
    pub fn from_default() -> Result<Self> {
        Rapl::new(None, None, None)
    }

    /// Reads a snapshot of current energy counters for all domains.
    ///
    /// Returns a `Snapshot` containing the energy in microjoules.
    fn read_snapshot(&self) -> Result<Snapshot> {
        trace!(
            "Reading energy snapshot from {} RAPL domain(s)",
            self.domains.len()
        );

        let mut map = HashMap::with_capacity(self.domains.len());

        for domain in &self.domains {
            trace!(
                "Reading domain: type={:?} socket={}",
                domain.domain_type, domain.socket
            );

            let val_uj = read_energy(domain)?;
            trace!("Energy read: {} µJ", val_uj);

            map.insert((domain.domain_type, domain.socket), val_uj);
        }

        Ok(Snapshot { metrics: map })
    }
}

impl MetricReader for Rapl {
    type Type = Snapshot;
    type Error = RaplError;

    async fn measure(&mut self) -> Result<()> {
        let new_snapshot = self.read_snapshot()?;

        let mut last = self.last_snapshot.lock().await;

        if let Some(prev) = last.as_ref() {
            let metrics = compute_measurement_from_snapshots(&self.domains, prev, &new_snapshot)?;

            let mut counters = self.current_counters.lock().await;
            *counters += Snapshot { metrics };
        }

        *last = Some(new_snapshot);
        Ok(())
    }

    fn get_sensors(&self) -> Result<Sensors> {
        trace!("Building RAPL sensor list");

        let sensors = self
            .domains
            .iter()
            .map(|domain| {
                trace!("Registering sensor: {}", domain.get_name());
                Sensor {
                    name: domain.get_name(),
                    unit: MICRO_JOULE_UNIT,
                    source: Self::get_name().to_lowercase(),
                }
            })
            .collect();

        Ok(sensors)
    }

    async fn retrieve(&mut self) -> Result<Snapshot> {
        let mut lock = self.current_counters.lock().await;
        Ok(std::mem::take(&mut *lock))
    }

    fn get_name() -> &'static str {
        POWERCAP_SOURCE_NAME
    }

    fn to_metrics(&self, result: Self::Type) -> Metrics {
        result.into()
    }

    async fn init(&mut self) -> Result<()> {
        check_rapl_access(&self.rapl_path)?;

        let mut ticker = if let Some(interval) = self.poll_interval {
            Interval::new_interval(interval)?
        } else {
            return Ok(());
        };

        let domains = self.domains.clone();
        let last_snapshot = Arc::clone(&self.last_snapshot);
        let current_counters = Arc::clone(&self.current_counters);

        let handle = tokio::spawn(async move {
            loop {
                ticker.next().await;
                trace!("Rapl polled");

                let mut last_guard = match last_snapshot.try_lock() {
                    Ok(guard) => guard,
                    Err(_) => {
                        trace!("RAPL measure skipped: previous measurement still running");
                        continue;
                    }
                };

                let new_snapshot = {
                    let mut map = HashMap::with_capacity(domains.len());
                    for domain in &domains {
                        let val_uj = read_energy(domain)?;
                        map.insert((domain.domain_type, domain.socket), val_uj);
                    }
                    Snapshot { metrics: map }
                };

                if let Some(prev) = last_guard.as_ref() {
                    let metrics =
                        compute_measurement_from_snapshots(&domains, prev, &new_snapshot)?;

                    let mut counters = current_counters.lock().await;
                    *counters += Snapshot { metrics };
                }

                *last_guard = Some(new_snapshot);
            }
        });

        self.handle = Some(handle);
        Ok(())
    }

    async fn join(&mut self) -> Result<()> {
        if let Some(handle) = self.handle.take() {
            if handle.is_finished() {
                if let Ok(res) = handle.await {
                    return res;
                }
            } else {
                handle.abort();
            }
        }
        Ok(())
    }

    async fn reset(&mut self) -> Result<()> {
        std::mem::take(&mut *self.current_counters.lock().await);
        self.last_snapshot.lock().await.take();
        Ok(())
    }
}

/// Resolves the RAPL base path from configuration and environment.
fn rapl_base_path(config_override: Option<&str>) -> String {
    if let Some(path) = config_override {
        return path.to_string();
    }

    if let Ok(env_path) = env::var("JOULE_PROFILER_RAPL_PATH") {
        return env_path;
    }

    DEFAULT_RAPL_PATH.to_string()
}

/// Checks if the operating system is Linux.
fn check_os() -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        Ok(())
    }

    #[cfg(not(target_os = "linux"))]
    {
        let os = std::env::consts::OS;
        Err(RaplError::UnsupportedOS(os.to_string()).into())
    }
}

/// Check if the program can read RAPL powercap files
fn check_rapl_access(base: &str) -> Result<()> {
    debug!("Checking RAPL access using base path: {}", base);
    let path = Path::new(base);

    let entries = fs::read_dir(path).map_err(|e| match e.kind() {
        ErrorKind::PermissionDenied => RaplError::InsufficientPermissions,
        _ => e.into(),
    })?;

    for entry in entries.flatten() {
        let energy_path = entry.path().join("energy_uj");
        if energy_path.exists() {
            fs::read_to_string(&energy_path).map_err(|e| {
                if e.kind() == ErrorKind::PermissionDenied {
                    RaplError::InsufficientPermissions
                } else {
                    e.into()
                }
            })?;
            return Ok(());
        }
    }

    Err(RaplError::RaplNotAvailable(format!(
        "{} is not an RAPL folder",
        base
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rapl_base_path_uses_override() {
        let path = rapl_base_path(Some("/custom/path"));
        assert_eq!(path, "/custom/path");
    }
}
