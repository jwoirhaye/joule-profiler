use std::{
    collections::{HashMap, HashSet},
    fs::read_to_string,
    time::Duration,
};

use anyhow::Result;
use log::{debug, error, info, trace};

use crate::{
    error::JouleProfilerError,
    source::{
        Metric, MetricReader, MetricSource, Metrics, Sensor, SourceResult,
        rapl::{
            domain::{RaplDomain, get_domains},
            snapshot::{EnergySnapshot, compute_measurement_from_snapshots},
        },
    },
    util::time::{duration_from_hz, get_timestamp},
};

pub mod domain;
pub mod snapshot;

pub fn init_rapl(
    rapl_path: Option<&str>,
    sockets: Option<&HashSet<u32>>,
    polling_rate_hz: Option<u64>,
) -> Result<MetricSource> {
    let domains = get_domains(rapl_path, sockets)?;
    let rapl = Rapl::new(domains, polling_rate_hz);
    Ok(MetricSource::Rapl(rapl))
}

#[derive(Clone, Debug)]
pub struct Rapl {
    domains: Vec<RaplDomain>,
    measures: Vec<HashMap<String, u64>>,
    last_measure: Option<EnergySnapshot>,
    measure_counters: HashMap<String, u64>,
    poll_interval: Option<Duration>,
    count: u64,
}

impl MetricReader for Rapl {
    fn measure(&mut self) -> Result<()> {
        self.count += 1;
        trace!("Starting RAPL measurement");

        let new_measure = match self.read_snapshot() {
            Ok(snap) => snap,
            Err(e) => {
                error!("Failed to read snapshot: {:?}", e);
                return Err(e);
            }
        };

        if let Some(old) = self.last_measure.take() {
            let diff = compute_measurement_from_snapshots(&self.domains, &old, &new_measure)?;
            for (k, v) in diff.iter() {
                *self.measure_counters.entry(k.clone()).or_insert(0) += *v;
                debug!("Updated counter {} = {}", k, self.measure_counters[k]);
            }
        }

        self.last_measure = Some(new_measure);
        trace!("Measurement complete");
        Ok(())
    }

    fn phase(&mut self) -> Result<()> {
        info!("Starting a new phase");
        self.measure()?;
        let phase_counters = std::mem::take(&mut self.measure_counters);
        debug!("Phase counters: {:?}", phase_counters);
        self.measures.push(phase_counters);
        info!("Phase completed, stored counters");
        Ok(())
    }

    fn retrieve(&mut self) -> Result<SourceResult> {
        info!("Retrieving all measures");

        let measure_counters = std::mem::take(&mut self.measure_counters);

        if !measure_counters.is_empty() {
            debug!(
                "Adding remaining counters before retrieval: {:?}",
                measure_counters
            );
            self.measures.push(measure_counters);
        }

        let measures: Vec<Metrics> = self
            .measures
            .iter()
            .map(|measure| {
                let mut total_uj = 0;
                let mut has_psys = false;
                let mut map: Vec<Metric> = Vec::with_capacity(measure.len() + 1);

                for (domain_name, value) in measure.iter() {
                    map.push(Metric {
                        name: domain_name.clone(),
                        value: *value,
                        unit: "µJ".to_string(),
                        source: "powercap".to_string(),
                    });

                    if domain_name.starts_with("PSYS") {
                        total_uj = *value;
                        has_psys = true;
                    } else if !has_psys
                        && (domain_name.starts_with("PACKAGE") || domain_name.starts_with("DRAM"))
                    {
                        total_uj += *value;
                    }
                }

                map.push(Metric {
                    name: "total_µJ".to_string(),
                    value: total_uj,
                    unit: "µJ".to_string(),
                    source: "powercap".to_string(),
                });

                map
            })
            .collect();

        info!("Retrieved {} phases", measures.len());
        Ok(SourceResult {
            measures,
            count: self.count,
        })
    }

    fn get_sensors(&self) -> Result<Vec<Sensor>> {
        let sensors: Vec<Sensor> = self
            .domains
            .iter()
            .map(|domain| {
                let sensor_name = format!("{}_{}", domain.name.to_uppercase(), domain.socket);
                debug!("Discovered sensor: {}", sensor_name);
                Sensor {
                    name: sensor_name,
                    source: "powercap".to_string(),
                    unit: "µJ".to_string(),
                }
            })
            .collect();

        info!("Total sensors discovered: {}", sensors.len());
        Ok(sensors)
    }

    fn get_polling_interval(&self) -> Option<Duration> {
        self.poll_interval
    }

    fn get_name(&self) -> &'static str {
        "Powercap"
    }
}

impl Rapl {
    /// Initialize an RAPL metric source.
    pub fn new(domains: Vec<RaplDomain>, poll_interval: Option<u64>) -> Self {
        Rapl {
            domains,
            poll_interval: poll_interval.map(|hz| duration_from_hz(hz)),
            count: 0,
            measures: Vec::new(),
            last_measure: None,
            measure_counters: HashMap::new(),
        }
    }

    /// Read all RAPL domains energy consumption counter.
    pub fn read_snapshot(&self) -> Result<EnergySnapshot> {
        trace!(
            "Reading energy snapshot from {} domains",
            self.domains.len()
        );

        let mut map = HashMap::with_capacity(self.domains.len());

        for domain in &self.domains {
            let val_str = read_to_string(&domain.path).map_err(|e| {
                error!(
                    "Failed to read energy from domain '{}': {:?}",
                    domain.name, e
                );
                if e.kind() == std::io::ErrorKind::PermissionDenied {
                    JouleProfilerError::InsufficientPermissions
                } else {
                    JouleProfilerError::RaplReadError(format!(
                        "Failed to read energy from domain '{}': {}",
                        domain.name, e
                    ))
                }
            })?;

            let val_uj: u64 = val_str.trim().parse().map_err(|e| {
                error!(
                    "Invalid energy value '{}' in domain '{}': {:?}",
                    val_str.trim(),
                    domain.name,
                    e
                );
                JouleProfilerError::ParseEnergyError(format!(
                    "Invalid energy value '{}' in domain '{}': {}",
                    val_str.trim(),
                    domain.name,
                    e
                ))
            })?;

            trace!("Read {} µJ from domain '{}'", val_uj, domain.name);
            map.insert(domain.path.to_string_lossy().to_string(), val_uj);
        }

        let timestamp_us = get_timestamp();
        trace!("Snapshot timestamp: {}", timestamp_us);

        Ok(EnergySnapshot {
            energies_uj: map,
            timestamp_us,
        })
    }
}
