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
//! use joule_profiler::sources::Rapl;
//! use joule_profiler::reader::MetricReader;
//! use std::collections::HashSet;
//!
//! // Initialize a RAPL reader (no polling, monitoring all sockets)
//! let mut rapl = Rapl::try_new("/sys/devices/virtual/powercap/intel-rapl", None, None).unwrap();
//!
//! // Measure and update internal counters
//! rapl.measure().unwrap();
//!
//! // Retrieve available sensors
//! let sensors = rapl.get_sensors().unwrap();
//!
//! // Retrieve collected counters
//! let counters = rapl.retrieve().unwrap();
//! ```
//!
//! # Errors
//!
//! All RAPL operations return a [`RaplError`]. Possible errors include:
//! - [`RaplError::RaplNotAvailable`] - no RAPL domains found at the specified path.
//! - [`RaplError::InsufficientPermissions`] - requires elevated privileges to read powercap files.
//! - [`RaplError::UnsupportedOS`] - only Linux is supported.
//! - [`RaplError::RaplReadError`] or [`RaplError::InvalidRaplPath`] - problems reading counters or invalid paths.

use std::{
    collections::{HashMap, HashSet},
    env, fs,
    io::ErrorKind,
    path::Path,
    time::Duration,
};

use futures::StreamExt;
use log::{debug, error, info, trace};
use tokio_timerfd::Interval;

use crate::{
    config::{Command, Config},
    core::{
        sensor::{Sensor, Sensors},
        source::reader::MetricReader,
    },
    sources::rapl::{
        domain::{RaplDomain, get_domains, read_energy},
        error::RaplError,
        snapshot::{Snapshot, compute_measurement_from_snapshots},
    },
};

mod domain;
pub(crate) mod error;
mod snapshot;

const POWERCAP_SOURCE_NAME: &str = "Powercap";

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
/// - `ticker`: Optional periodic polling interval. If set, [`Self::scheduler`] will trigger
///   measurements at this interval.
/// - `current_counters`: Latest energy counters collected by this reader. Updated by
///   [`Self::measure`] and returned by [`Self::retrieve`].
/// - `last_snapshot`: Last snapshot read from RAPL domains, used to compute the energy delta
///   between measurements.
#[derive(Default)]
pub struct Rapl {
    domains: Vec<RaplDomain>,

    ticker: Option<Interval>,

    current_counters: Snapshot,

    last_snapshot: Option<Snapshot>,
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
    pub fn try_new(
        rapl_path: &str,
        sockets: Option<&HashSet<u32>>,
        polling_rate_s: Option<f64>,
    ) -> Result<Self> {
        debug!("Initializing RAPL reader");
        trace!("rapl_path={}", rapl_path);
        trace!("sockets={:?}", sockets);
        trace!("polling_rate_s={:?}", polling_rate_s);

        check_os()?;
        check_rapl(rapl_path)?;

        let domains = get_domains(rapl_path, sockets)?;
        info!("Discovered {} RAPL domain(s)", domains.len());

        let poll_interval = polling_rate_s.map(Duration::from_secs_f64);

        let ticker = if let Some(duration) = poll_interval {
            debug!("Enabling RAPL polling every {:?}", duration);
            let timerfd_interval = Interval::new_interval(duration)?;
            Some(timerfd_interval)
        } else {
            debug!("RAPL polling disabled (on-demand only)");
            None
        };

        Ok(Rapl::new(domains, ticker))
    }

    /// Create a new RAPL instance with domains and optional ticker.
    fn new(domains: Vec<RaplDomain>, ticker: Option<Interval>) -> Self {
        trace!(
            "Creating Rapl instance (domains={}, ticker={})",
            domains.len(),
            ticker.is_some()
        );

        Self {
            domains,
            ticker,
            ..Default::default()
        }
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

impl TryFrom<&Config> for Rapl {
    type Error = RaplError;

    /// Initialize a `Rapl` reader from a [`Config`] object.
    ///
    /// Automatically resolves the base path and polling settings.
    fn try_from(config: &Config) -> Result<Self> {
        let base_path = rapl_base_path(config.rapl_path.as_deref());

        let (sockets, rapl_polling) = match &config.command {
            Command::Profile(profile_config) => {
                check_root_rights(&base_path)?;
                (profile_config.sockets.as_ref(), profile_config.rapl_polling)
            }
            Command::ListSensors(_) => (None, None),
        };

        let rapl = Rapl::try_new(&base_path, sockets, rapl_polling)?;
        Ok(rapl)
    }
}

impl MetricReader for Rapl {
    type Type = Snapshot;
    type Error = RaplError;

    fn measure(&mut self) -> Result<()> {
        trace!("RAPL measure() called");

        let new_snapshot = self.read_snapshot()?;

        if let Some(last_snapshot) = &self.last_snapshot {
            trace!("Computing delta from previous snapshot");

            let metrics =
                compute_measurement_from_snapshots(&self.domains, last_snapshot, &new_snapshot)?;

            trace!("Computed {} metric(s)", metrics.len());
            self.current_counters += Snapshot::new(metrics);
        } else {
            trace!("First snapshot recorded (no delta)");
            self.last_snapshot = Some(new_snapshot);
        }

        Ok(())
    }

    async fn scheduler(&mut self) -> Result<()> {
        if let Some(ticker) = &mut self.ticker {
            trace!("Waiting for RAPL scheduler tick");
            ticker.next().await;
            trace!("RAPL scheduler tick fired");
            self.measure()?;
        }
        Ok(())
    }

    fn get_sensors(&self) -> Result<Sensors> {
        trace!("Building RAPL sensor list");

        let sensors = self
            .domains
            .iter()
            .map(|domain| {
                trace!("Registering sensor: {}", domain.get_name());

                Sensor::new(
                    domain.get_name(),
                    "µJ".to_string(),
                    Self::get_name().to_lowercase(),
                )
            })
            .collect();

        Ok(sensors)
    }

    fn retrieve(&mut self) -> Result<Self::Type> {
        trace!(
            "Retrieving RAPL counters ({} entries)",
            self.current_counters.metrics.len()
        );

        let counters = std::mem::take(&mut self.current_counters);
        Ok(counters)
    }

    fn get_name() -> &'static str {
        POWERCAP_SOURCE_NAME
    }
}

/// Checks if the RAPL interface is available at the given base path.
fn check_rapl(base: &str) -> Result<()> {
    debug!("Checking RAPL base path: {}", base);
    trace!("Validating filesystem entry");

    let path = Path::new(base);

    if !path.exists() {
        error!("RAPL path does not exist: {}", base);
        return Err(RaplError::RaplNotAvailable(base.into()));
    }

    if !path.is_dir() {
        error!("RAPL path is not a directory: {}", base);
        return Err(RaplError::InvalidRaplPath(base.into()));
    }

    info!("RAPL interface found at {}", base);
    Ok(())
}

/// Check if the program can read RAPL powercap files
fn check_root_rights(base: &str) -> Result<()> {
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

    Err(RaplError::RaplNotAvailable(base.into()))
}

/// Resolves the RAPL base path from configuration and environment.
fn rapl_base_path(config_override: Option<&str>) -> String {
    if let Some(path) = config_override {
        return path.to_string();
    }

    if let Ok(env_path) = env::var("JOULE_PROFILER_RAPL_PATH") {
        return env_path;
    }

    let default_path = "/sys/devices/virtual/powercap/intel-rapl";
    default_path.to_string()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rapl_base_path_uses_override() {
        let path = rapl_base_path(Some("/custom/path"));
        assert_eq!(path, "/custom/path");
    }
}
