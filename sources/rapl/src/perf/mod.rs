use std::{collections::HashSet, fs, io::ErrorKind};

use joule_profiler_core::{
    sensor::{Sensor, Sensors},
    source::MetricReader,
    types::Metric,
};
use log::{info, trace};

use crate::{
    MICRO_JOULE_UNIT, Result,
    error::{PerfParanoidError, RaplError},
    perf::{
        compute::compute_measurement_from_snapshots,
        domain::{PerfRaplDomain, discover_domains_and_open_counters},
    },
    snapshot::Snapshot,
    util::check_os,
};

mod compute;
mod domain;
mod socket;

/// Default sysfs path for perf RAPL counters.
const PERF_RAPL_PATH: &str = "/sys/bus/event_source/devices/power";

/// perf RAPL source name for this metric reader.
const PERF_SOURCE_NAME: &str = "RAPL (perf_event)";

/// Path to check the kernel perf_event_paranoid level.
const PERF_PARANOID_PATH: &str = "/proc/sys/kernel/perf_event_paranoid";

/// RAPL energy measurement source using Linux perf events.
///
/// Provides energy readings per RAPL domain (e.g., package, core, dram).
pub struct Rapl {
    /// Discovered RAPL domains and associated counters.
    domains: Vec<PerfRaplDomain>,

    /// Accumulated metrics from measurements.
    current_counters: Snapshot,

    /// Snapshot of the last measurement for delta calculations.
    last_snapshot: Option<Snapshot>,
}

impl Rapl {
    /// Initialize a new RAPL reader.
    ///
    /// # Arguments
    ///
    /// * `rapl_path` - Optional path to RAPL sysfs directory.
    /// * `sockets_spec` - Optional set of socket IDs to restrict measurement.
    ///
    /// # Returns
    ///
    /// A `Rapl` instance with discovered domains.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - OS is unsupported
    /// - RAPL counters cannot be read
    /// - Perf PMU type cannot be retrieved
    pub fn new(rapl_path: Option<&str>, sockets_spec: Option<HashSet<u32>>) -> Result<Self> {
        check_os()?;

        let paranoid_level = read_paranoid_level()?;
        trace!("Perf paranoid level set to {}", paranoid_level);

        let rapl_path = rapl_path.unwrap_or(PERF_RAPL_PATH);
        trace!(
            "Attempting to initialize RAPL reader: rapl_path={}, sockets={:?}",
            rapl_path, sockets_spec
        );

        let pmu_type = read_pmu_type()?;
        let domains =
            discover_domains_and_open_counters(pmu_type, rapl_path, sockets_spec.as_ref())?;
        info!("Discovered {} RAPL domain(s)", domains.len());

        Ok(Self {
            domains,
            current_counters: Default::default(),
            last_snapshot: None,
        })
    }

    /// Check if perf events access is allowed.
    ///
    /// # Errors
    ///
    /// Returns an error if PMU type cannot be read.
    pub fn check_perf_access() -> Result<()> {
        read_pmu_type()?;
        Ok(())
    }

    /// Enable counters for all domains.
    fn enable_domains_counter(&self) {
        for domain in &self.domains {
            domain.enable_counter();
        }
    }

    /// Reset counters for all domains.
    fn reset_domains_counter(&self) {
        for domain in &self.domains {
            domain.reset_counter();
        }
    }

    /// Read the current counter values for all domains.
    ///
    /// Returns a `Snapshot` containing domain metrics.
    fn read_domains_counter(&mut self) -> Result<Snapshot> {
        let metrics = self
            .domains
            .iter_mut()
            .map(|domain| {
                let value = domain.read_counter()?;
                let domain_index = (domain.domain_type, domain.socket);
                Ok((domain_index, value))
            })
            .collect::<Result<_>>()?;

        Ok(Snapshot { metrics })
    }
}

impl MetricReader for Rapl {
    type Type = Snapshot;
    type Error = RaplError;

    /// Perform a measurement and accumulate metrics.
    ///
    /// Computes delta between last and current snapshot.
    async fn measure(&mut self) -> Result<()> {
        let new_snapshot = self.read_domains_counter()?;
        if let Some(last_snapshot) = &self.last_snapshot {
            let metrics =
                compute_measurement_from_snapshots(&self.domains, last_snapshot, &new_snapshot)?;
            self.current_counters += metrics;
        }
        self.last_snapshot = Some(new_snapshot);
        Ok(())
    }

    /// Retrieve accumulated metrics since the last reset.
    async fn retrieve(&mut self) -> Result<Self::Type> {
        trace!(
            "Retrieving RAPL counters ({} entries)",
            self.current_counters.metrics.len()
        );
        let counters = std::mem::take(&mut self.current_counters);
        Ok(counters)
    }

    /// Return the list of available sensors for this source.
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
                    source: Self::get_name().to_string(),
                }
            })
            .collect();

        Ok(sensors)
    }

    /// Return the source name.
    fn get_name() -> &'static str {
        PERF_SOURCE_NAME
    }

    /// Convert a `Snapshot` into a list of `Metric`s.
    fn to_metrics(&self, snapshot: Self::Type) -> joule_profiler_core::types::Metrics {
        self.domains
            .iter()
            .filter_map(|domain| {
                let domain_index = (domain.domain_type, domain.socket);
                if let Some(metric) = snapshot.metrics.get(&domain_index) {
                    let joules = domain.compute_scale(*metric);
                    let micro_joules = joules_to_micro_joules(joules);
                    Some(Metric {
                        name: domain.domain_type.to_string_socket(domain.socket),
                        value: micro_joules,
                        unit: MICRO_JOULE_UNIT,
                        source: PERF_SOURCE_NAME.to_string(),
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    /// Initialize the source (enable counters).
    async fn init(&mut self) -> Result<()> {
        self.enable_domains_counter();
        Ok(())
    }

    /// Reset the source (reset counters).
    async fn reset(&mut self) -> Result<()> {
        self.reset_domains_counter();
        Ok(())
    }
}

/// Read the PMU type from sysfs.
///
/// # Errors
///
/// Returns `RaplError` if the file cannot be read or parsed.
fn read_pmu_type() -> Result<u32> {
    let type_path = format!("{}/type", PERF_RAPL_PATH);
    fs::read_to_string(type_path)
        .map_err(|err| RaplError::RaplNotAvailable(format!("Failed to read perf PMU type {}", err)))
        .map(|pmu_type_str| pmu_type_str.trim().parse::<u32>())?
        .map_err(Into::into)
}

/// Read the perf_event_paranoid level from `/proc`.
///
/// # Errors
///
/// Returns `PerfParanoidError` if the file cannot be read or parsed.
fn read_paranoid_level() -> Result<u8> {
    fs::read_to_string(PERF_PARANOID_PATH)
        .map_err(|err| match err.kind() {
            ErrorKind::NotFound => PerfParanoidError::NotFound,
            ErrorKind::PermissionDenied => PerfParanoidError::PermissionDenied(err.to_string()),
            _ => PerfParanoidError::IoError(err),
        })
        .map(|paranoid_level_str| paranoid_level_str.trim().parse::<u8>())?
        .map_err(|err| PerfParanoidError::ParseParanoidLevelError(err).into())
}

/// Convert joules to microjoules.
#[inline]
pub fn joules_to_micro_joules(joules: f64) -> u64 {
    (joules * 1_000_000.0) as u64
}
