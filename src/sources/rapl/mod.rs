use std::{
    collections::{HashMap, HashSet},
    fs::{self},
    path::Path,
    time::Duration,
};

use anyhow::Result;
use log::{debug, error, info, trace};

use crate::{
    core::{
        metric::{Metric, Metrics},
        sensor::Sensor,
        source::MetricReader,
    },
    error::JouleProfilerError,
    sources::{
        MetricSourceType,
        rapl::{
            domain::{RaplDomain, get_domains, rapl_base_path, read_energy},
            snapshot::{Snapshot, compute_measurement_from_snapshots},
        },
    },
    util::platform::check_os,
};

pub mod domain;
pub mod snapshot;

pub fn init_rapl(
    rapl_path: Option<&str>,
    sockets: Option<&HashSet<u32>>,
    polling_rate_s: Option<f64>,
) -> Result<MetricSourceType> {
    check_os()?;

    let base_path = rapl_base_path(rapl_path);
    check_rapl(&base_path)?;

    let domains = get_domains(&base_path, sockets)?;
    let rapl = Rapl::new(domains, polling_rate_s);
    Ok(MetricSourceType::Rapl(rapl))
}

/// Checks if the RAPL interface is available at the given base path.
pub fn check_rapl(base: &str) -> Result<()> {
    debug!("Checking RAPL base path: {}", base);

    let path = Path::new(base);

    if !path.exists() {
        error!("RAPL path does not exist: {}", base);
        return Err(JouleProfilerError::RaplNotAvailable(base.into()).into());
    }

    if !path.is_dir() {
        error!("RAPL path is not a directory: {}", base);
        return Err(JouleProfilerError::InvalidRaplPath(base.into()).into());
    }

    if let Err(e) = fs::read_dir(path) {
        if e.kind() == std::io::ErrorKind::PermissionDenied {
            error!("Permission denied accessing RAPL path");
            return Err(JouleProfilerError::InsufficientPermissions.into());
        }
        return Err(e.into());
    }

    info!("RAPL interface found at {}", base);
    Ok(())
}

#[derive(Clone, Debug)]
pub struct Rapl {
    domains: Vec<RaplDomain>,
    measures: Vec<HashMap<String, u64>>,
    last_measure: Option<Snapshot>,
    measure_counters: HashMap<String, u64>,
    poll_interval: Option<Duration>,
}

impl MetricReader<Snapshot> for Rapl {
    fn measure(&mut self) -> Result<Snapshot> {
        trace!("Starting RAPL measurement");

        let new_measure = self.read_snapshot()?;

        // if let Some(old) = self.last_measure.take() {
        //     let diff = compute_measurement_from_snapshots(&self.domains, &old, &new_measure)?;
        //     for (k, v) in diff.iter() {
        //         *self.measure_counters.entry(k.clone()).or_insert(0) += *v;
        //         debug!("Updated counter {} = {}", k, self.measure_counters[k]);
        //     }
        // }

        
        trace!("Measurement complete");
        Ok(new_measure)
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
    
    fn compute_measures(&self, new: Snapshot, old: Snapshot) -> Result<Snapshot> {
        let metrics = compute_measurement_from_snapshots(&self.domains, &old, &new)?;
        let snapshot = Snapshot { metrics };
        Ok(snapshot)
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
        }
    }

    pub fn read_snapshot(&self) -> Result<Snapshot> {
        trace!(
            "Reading energy snapshot from {} domains",
            self.domains.len()
        );

        let mut map = HashMap::with_capacity(self.domains.len());

        for domain in &self.domains {
            let val_uj = read_energy(&domain)?;
            map.insert(domain.path.to_string_lossy().to_string(), val_uj);
        }

        Ok(Snapshot { metrics: map })
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

        assert_eq!(snapshot.metrics.len(), 1);
        assert_eq!(*snapshot.metrics.values().next().unwrap(), 12345);
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

        assert_eq!(rapl.measures.len(), 1);
        assert!(rapl.measure_counters.is_empty());

        let phase = &rapl.measures[0];
        assert_eq!(*phase.values().next().unwrap(), 50);
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
}
