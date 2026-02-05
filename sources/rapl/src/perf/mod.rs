use std::{fs, io::ErrorKind};

use joule_profiler_core::{
    sensor::{Sensor, Sensors},
    source::MetricReader,
    sys::is_root,
    types::Metric,
};
use log::{info, trace};

use crate::{
    MICRO_JOULE_UNIT, Result,
    domain::socket::parse_sockets_spec,
    error::{PerfParanoidError, RaplError},
    perf::{
        domain::{PerfRaplDomain, discover_domains_and_open_counters},
        snapshot::{PerfSnapshot, compute_measurement_from_snapshots},
    },
    util::check_os,
};

mod domain;
mod snapshot;
mod socket;

const PERF_RAPL_PATH: &str = "/sys/bus/event_source/devices/power";
const PERF_SOURCE_NAME: &str = "perf";
const PERF_PARANOID_PATH: &str = "/proc/sys/kernel/perf_event_paranoid";

#[derive(Default)]
pub struct Rapl {
    domains: Vec<PerfRaplDomain>,

    current_counters: PerfSnapshot,

    last_snapshot: Option<PerfSnapshot>,
}

impl Rapl {
    pub fn new(rapl_path: Option<&str>, sockets_spec: Option<&str>) -> Result<Self> {
        check_os()?;

        let paranoid_level = read_paranoid_level()?;
        trace!("Perf paranoid level set to {}", paranoid_level);

        if paranoid_level > 0 && !is_root() {
            return Err(PerfParanoidError::LevelTooHigh(paranoid_level).into());
        }

        let rapl_path = rapl_path.unwrap_or(PERF_RAPL_PATH);
        trace!(
            "Attempting to initialize RAPL reader: rapl_path={}, sockets={:?}",
            rapl_path, sockets_spec
        );

        let sockets = parse_sockets_spec(sockets_spec);

        let pmu_type = read_pmu_type()?;
        let domains = discover_domains_and_open_counters(pmu_type, rapl_path, sockets.as_ref())?;
        info!("Discovered {} RAPL domain(s)", domains.len());

        Ok(Self {
            domains,
            ..Default::default()
        })
    }

    pub fn check_perf_access() -> Result<()> {
        read_pmu_type()?;
        Ok(())
    }

    fn enable_domains_counter(&self) {
        for domain in &self.domains {
            domain.enable_counter();
        }
    }

    fn reset_domains_counter(&self) {
        for domain in &self.domains {
            domain.reset_counter();
        }
    }

    fn read_domains_counter(&mut self) -> Result<PerfSnapshot> {
        let metrics = self
            .domains
            .iter_mut()
            .map(|domain| {
                let value = domain.read_counter()?;
                let domain_index = (domain.domain_type, domain.socket);
                Ok((domain_index, value))
            })
            .collect::<Result<_>>()?;

        Ok(PerfSnapshot { metrics })
    }
}

impl MetricReader for Rapl {
    type Type = PerfSnapshot;

    type Error = RaplError;

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

    async fn retrieve(&mut self) -> Result<Self::Type> {
        trace!(
            "Retrieving RAPL counters ({} entries)",
            self.current_counters.metrics.len()
        );
        let counters = std::mem::take(&mut self.current_counters);
        Ok(counters)
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

    fn get_name() -> &'static str {
        PERF_SOURCE_NAME
    }

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
                        unit: MICRO_JOULE_UNIT.to_string(),
                        source: PERF_SOURCE_NAME.to_lowercase(),
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    async fn init(&mut self) -> Result<()> {
        self.enable_domains_counter();
        Ok(())
    }

    async fn reset(&mut self) -> Result<()> {
        self.reset_domains_counter();
        Ok(())
    }
}

fn read_pmu_type() -> Result<u32> {
    let type_path = format!("{}/type", PERF_RAPL_PATH);
    fs::read_to_string(type_path)
        .map_err(|err| RaplError::RaplNotAvailable(format!("Failed to read perf PMU type {}", err)))
        .map(|pmu_type_str| pmu_type_str.trim().parse::<u32>())?
        .map_err(Into::into)
}

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

#[inline]
pub fn joules_to_micro_joules(joules: f64) -> u64 {
    (joules * 1_000_000.0) as u64
}
