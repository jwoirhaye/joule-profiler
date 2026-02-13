use std::{
    collections::{HashMap, HashSet},
    fs,
    io::ErrorKind,
};

use joule_profiler_core::{
    sensor::{Sensor, Sensors},
    source::MetricReader,
    types::{Metric, Metrics},
};
use log::{info, trace};
use perf_event::GroupData;

use crate::{
    MICRO_JOULE_UNIT, Result,
    error::{PerfParanoidError, RaplError},
    perf::{
        compute::{compute_measurement_from_snapshots, joules_to_micro_joules},
        domain::discover_domains_and_open_counters,
        socket::Socket,
    },
    snapshot::Snapshot,
    util::check_os,
};

mod compute;
mod domain;
mod event;
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
    sockets: Vec<Socket>,

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
    pub fn new(sockets_spec: Option<HashSet<u32>>) -> Result<Self> {
        check_os()?;

        let paranoid_level = read_paranoid_level()?;
        trace!("Perf paranoid level set to {}", paranoid_level);

        trace!(
            "Attempting to initialize RAPL reader: sockets={:?}",
            sockets_spec
        );

        let sockets = discover_domains_and_open_counters(sockets_spec.as_ref()).map_err(|err| {
            if let RaplError::PerfParanoid(_) = err
                && paranoid_level > 0
            {
                PerfParanoidError::ParanoidLevelTooHigh(paranoid_level).into()
            } else {
                err
            }
        })?;

        info!("Discovered {} socket(s)", sockets.len());

        Ok(Self {
            sockets,
            current_counters: Default::default(),
            last_snapshot: None,
        })
    }

    /// Check if perf events access is allowed.
    pub fn check_perf_access() -> Result<()> {
        read_pmu_type()?;
        Ok(())
    }

    /// Read the current counter values for all domains.
    ///
    /// Returns a `Snapshot` containing domain metrics.
    fn read_domains_counter(&mut self) -> Result<Snapshot> {
        let mut metrics = HashMap::new();

        for socket in &mut self.sockets {
            let group_data: GroupData = socket.group.read()?;

            for domain in &socket.domains {
                let domain_index = (domain.domain_type, socket.id);

                let value = group_data
                    .get(&domain.event_counter.counter)
                    .map(|d| d.value())
                    .unwrap_or(0);

                metrics.insert(domain_index, value);
            }
        }

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
                compute_measurement_from_snapshots(&self.sockets, last_snapshot, &new_snapshot)?;
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
            .sockets
            .iter()
            .flat_map(|socket| {
                socket.domains.iter().map(|domain| {
                    let domain_name = domain.get_name(socket.id);
                    trace!("Registering sensor: {}", domain_name);
                    Sensor {
                        name: domain_name,
                        unit: MICRO_JOULE_UNIT,
                        source: Self::get_name().to_string(),
                    }
                })
            })
            .collect();

        Ok(sensors)
    }

    /// Return the source name.
    fn get_name() -> &'static str {
        PERF_SOURCE_NAME
    }

    /// Convert a Snapshot into Metrics.
    fn to_metrics(&self, snapshot: Self::Type) -> Metrics {
        self.sockets
            .iter()
            .flat_map(|socket| {
                socket.domains.iter().filter_map(|domain| {
                    let domain_index = (domain.domain_type, socket.id);
                    if let Some(metric) = snapshot.metrics.get(&domain_index) {
                        let joules = domain.compute_scale(*metric);
                        let micro_joules = joules_to_micro_joules(joules);
                        Some(Metric {
                            name: domain.domain_type.to_string_socket(socket.id),
                            value: micro_joules,
                            unit: MICRO_JOULE_UNIT,
                            source: PERF_SOURCE_NAME.to_string(),
                        })
                    } else {
                        None
                    }
                })
            })
            .collect()
    }

    /// Enable the perf_event counters.
    async fn init(&mut self, _: i32) -> Result<()> {
        self.sockets
            .iter_mut()
            .try_for_each(|socket| socket.group.enable().map_err(RaplError::from))?;
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
