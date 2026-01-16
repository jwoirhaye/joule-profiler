use std::{
    collections::{HashMap, HashSet},
    fs::read_to_string,
    time::Duration,
};

use anyhow::Result;
use log::{debug, error, info, trace};
use tokio::time::Instant;

use crate::{
    error::JouleProfilerError,
    source::{
        Metric, MetricReader, MetricSource, Metrics, Sensor, SourceResult,
        rapl::{
            domain::{RaplDomain, get_domains},
            snapshot::{EnergySnapshot, compute_measurement_from_snapshots},
        },
    },
    util::time::get_timestamp,
};

pub mod domain;
pub mod snapshot;

pub fn init_rapl(
    rapl_path: Option<&str>,
    sockets: Option<&HashSet<u32>>,
    polling_rate_s: Option<f64>,
) -> Result<MetricSource> {
    let domains = get_domains(rapl_path, sockets)?;
    let rapl = Rapl::new(domains, polling_rate_s);
    Ok(MetricSource::Rapl(rapl))
}

#[derive(Clone, Debug)]
pub struct Rapl {
    domains: Vec<RaplDomain>,
    measures: Vec<HashMap<String, u64>>,
    last_measure: Option<EnergySnapshot>,
    measure_counters: HashMap<String, u64>,
    poll_interval: Option<Duration>,

    /// Number of snapshots taken
    count: u64,

    /// Total elapsed time between snapshots
    total_elapsed: Duration,

    /// Monotonic timestamp of last snapshot
    last_instant: Option<Instant>,
}

impl MetricReader for Rapl {
    fn measure(&mut self) -> Result<()> {
        trace!("Starting RAPL measurement");

        let new_measure = self.read_snapshot()?;

        let now = Instant::now();
        if let Some(last) = self.last_instant {
            self.total_elapsed += now.duration_since(last);
        }
        self.last_instant = Some(now);
        self.count += 1;

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

        let remaining = std::mem::take(&mut self.measure_counters);
        if !remaining.is_empty() {
            debug!("Adding remaining counters: {:?}", remaining);
            self.measures.push(remaining);
        }

        let measures: Vec<Metrics> = self
            .measures
            .iter()
            .map(|measure| {
                let mut metrics = Vec::with_capacity(measure.len());
                for (domain_name, value) in measure.iter() {
                    metrics.push(Metric {
                        name: domain_name.clone(),
                        value: *value,
                        unit: "µJ".to_string(),
                        source: "powercap".to_string(),
                    });
                }
                metrics
            })
            .collect();

        let avg_delta_us = if self.count > 1 {
            self.total_elapsed.as_micros() / (self.count - 1) as u128
        } else {
            0
        };

        info!("Retrieved {} phases", measures.len());

        Ok(SourceResult {
            measures,
            count: self.count,
            measure_delta: avg_delta_us,
        })
    }

    fn get_sensors(&self) -> Result<Vec<Sensor>> {
        let sensors = self
            .domains
            .iter()
            .map(|domain| {
                let name = format!("{}_{}", domain.name.to_uppercase(), domain.socket);
                Sensor {
                    name,
                    source: "powercap".to_string(),
                    unit: "µJ".to_string(),
                }
            })
            .collect();

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
    pub fn new(domains: Vec<RaplDomain>, polling_rate_s: Option<f64>) -> Self {
        Rapl {
            domains,
            poll_interval: polling_rate_s.map(Duration::from_secs_f64),
            measures: Vec::new(),
            last_measure: None,
            measure_counters: HashMap::new(),
            count: 0,
            total_elapsed: Duration::ZERO,
            last_instant: None,
        }
    }

    pub fn read_snapshot(&self) -> Result<EnergySnapshot> {
        trace!(
            "Reading energy snapshot from {} domains",
            self.domains.len()
        );

        let mut map = HashMap::with_capacity(self.domains.len());

        for domain in &self.domains {
            let val_str = read_to_string(&domain.path).map_err(|e| {
                error!("Failed to read energy from {}: {:?}", domain.name, e);
                if e.kind() == std::io::ErrorKind::PermissionDenied {
                    JouleProfilerError::InsufficientPermissions
                } else {
                    JouleProfilerError::RaplReadError(e.to_string())
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

            map.insert(domain.path.to_string_lossy().to_string(), val_uj);
        }

        Ok(EnergySnapshot {
            energies_uj: map,
            timestamp_us: get_timestamp(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::write;
    use tempfile::tempdir;

    fn make_domain(name: &str, socket: u32, path: &std::path::Path) -> RaplDomain {
        RaplDomain {
            name: name.to_string(),
            socket,
            path: path.to_path_buf(),
            max_energy_uj: u32::MAX as u64,
        }
    }

    #[test]
    fn read_snapshot_reads_energy_and_timestamp() {
        let dir = tempdir().unwrap();
        let energy_file = dir.path().join("energy_uj");
        write(&energy_file, "12345").unwrap();

        let domain = make_domain("package", 0, &energy_file);
        let rapl = Rapl::new(vec![domain], None);

        let snapshot = rapl.read_snapshot().unwrap();

        assert_eq!(snapshot.energies_uj.len(), 1);
        assert_eq!(*snapshot.energies_uj.values().next().unwrap(), 12345);
        assert!(snapshot.timestamp_us > 0);
    }

    #[test]
    fn measure_accumulates_energy_diff() {
        let dir = tempdir().unwrap();
        let energy_file = dir.path().join("energy_uj");

        write(&energy_file, "100").unwrap();
        let domain = make_domain("package", 0, &energy_file);
        let mut rapl = Rapl::new(vec![domain], None);

        rapl.measure().unwrap();
        assert!(rapl.measure_counters.is_empty());

        write(&energy_file, "250").unwrap();
        rapl.measure().unwrap();

        let value = rapl.measure_counters.values().next().unwrap();
        assert_eq!(*value, 150);
    }

    #[test]
    fn phase_stores_and_resets_counters() {
        let dir = tempdir().unwrap();
        let energy_file = dir.path().join("energy_uj");

        write(&energy_file, "10").unwrap();
        let domain = make_domain("package", 0, &energy_file);
        let mut rapl = Rapl::new(vec![domain], None);

        rapl.measure().unwrap();
        write(&energy_file, "60").unwrap();
        rapl.phase().unwrap();

        assert_eq!(rapl.measures.len(), 1);
        assert!(rapl.measure_counters.is_empty());

        let phase = &rapl.measures[0];
        assert_eq!(*phase.values().next().unwrap(), 50);
    }

    #[test]
    fn retrieve_returns_all_phases_and_count() {
        let dir = tempdir().unwrap();
        let energy_file = dir.path().join("energy_uj");

        write(&energy_file, "0").unwrap();
        let domain = make_domain("package", 0, &energy_file);
        let mut rapl = Rapl::new(vec![domain], None);

        rapl.measure().unwrap();
        write(&energy_file, "100").unwrap();
        rapl.phase().unwrap();

        let result = rapl.retrieve().unwrap();

        assert_eq!(result.measures.len(), 1);
        assert_eq!(result.count, 2);

        let metrics = &result.measures[0];
        assert_eq!(metrics[0].unit, "µJ");
        assert_eq!(metrics[0].source, "powercap");
        assert_eq!(metrics[0].value, 100);
    }

    #[test]
    fn measure_handles_energy_counter_overflow() {
        use std::fs::write;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let energy_file = dir.path().join("energy_uj");

        let max_energy_range_uj = 1_000;

        write(&energy_file, "900").unwrap();

        let domain = RaplDomain {
            name: "package".to_string(),
            socket: 0,
            path: energy_file.clone(),
            max_energy_uj: max_energy_range_uj,
        };

        let mut rapl = Rapl::new(vec![domain], None);

        rapl.measure().unwrap();

        write(&energy_file, "100").unwrap();
        rapl.measure().unwrap();

        let value = rapl.measure_counters.values().next().unwrap();
        assert_eq!(*value, 200);
    }

    #[test]
    fn measure_delta_is_average_of_intervals() {
        let dir = tempdir().unwrap();
        let energy_file = dir.path().join("energy_uj");
        write(&energy_file, "0").unwrap();

        let domain = make_domain("package", 0, &energy_file);
        let mut rapl = Rapl::new(vec![domain], None);

        rapl.measure().unwrap();
        std::thread::sleep(std::time::Duration::from_micros(100));
        rapl.measure().unwrap();

        let result = rapl.retrieve().unwrap();
        assert!(result.measure_delta > 0);
    }
}
