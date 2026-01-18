use std::{
    collections::{HashMap, HashSet},
    fs::{self},
    path::Path,
    time::Duration,
};

use anyhow::Result;
use futures::StreamExt;
use log::{debug, error, info, trace};
use tokio_timerfd::Interval;

use crate::{
    config::{Command, Config},
    core::{
        sensor::{Sensor, Sensors},
        source::{GetSensorsTrait, MetricReader},
    },
    error::JouleProfilerError,
    sources::rapl::{
        domain::{RaplDomain, get_domains, rapl_base_path, read_energy},
        snapshot::{Snapshot, compute_measurement_from_snapshots},
    },
    util::platform::check_os,
};

pub mod domain;
pub mod snapshot;

pub fn init_rapl(
    rapl_path: Option<&str>,
    sockets: Option<&HashSet<u32>>,
    polling_rate_s: Option<f64>,
) -> Result<Rapl> {
    check_os()?;

    let base_path = rapl_base_path(rapl_path);
    check_rapl(&base_path)?;

    let domains = get_domains(&base_path, sockets)?;
    Rapl::new(domains, polling_rate_s)
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

#[derive(Default)]
pub struct Rapl {
    domains: Vec<RaplDomain>,
    ticker: Option<Interval>,
    poll_interval: Option<Duration>,
}

impl Clone for Rapl {
    fn clone(&self) -> Self {
        Self {
            domains: self.domains.clone(),
            ticker: None,
            poll_interval: self.poll_interval,
        }
    }
}

impl TryFrom<&Config> for Rapl {
    type Error = anyhow::Error;

    fn try_from(config: &Config) -> Result<Self, Self::Error> {
        let (sockets, rapl_polling) = match &config.mode {
            Command::Profile(profile_config) => {
                (profile_config.sockets.as_ref(), profile_config.rapl_polling)
            }
            Command::ListSensors(_) => (None, None),
        };

        let rapl = init_rapl(config.rapl_path.as_deref(), sockets, rapl_polling)?;
        Ok(rapl)
    }
}

impl GetSensorsTrait for Rapl {
    fn get_sensors(&self) -> Result<Sensors> {
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
}

impl MetricReader for Rapl {
    type Type = Snapshot;

    fn measure(&self) -> Result<Self::Type> {
        trace!("Starting RAPL measurement");
        Ok(self.read_snapshot()?)
    }

    fn compute_measures(&self, new: &Self::Type, old: Self::Type) -> Result<Self::Type> {
        let metrics = compute_measurement_from_snapshots(&self.domains, &old, new)?;
        let snapshot = Self::Type::new(metrics);
        Ok(snapshot)
    }

    async fn poll_loop(&mut self) -> Option<Result<()>> {
        if let Some(ticker) = &mut self.ticker {
            let _ = ticker.next().await?;
            Some(Ok(()))
        } else {
            None
        }
    }
}

impl Rapl {
    pub fn new(domains: Vec<RaplDomain>, polling_rate_s: Option<f64>) -> Result<Self> {
        let poll_interval = polling_rate_s.map(Duration::from_secs_f64);

        let ticker = if let Some(duration) = poll_interval {
            let timerfd_interval = Interval::new_interval(duration)?;
            Some(timerfd_interval)
        } else {
            None
        };

        Ok(Rapl {
            domains,
            poll_interval,
            ticker,
        })
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
        let rapl = Rapl::new(vec![domain], None).unwrap();

        let snapshot = rapl.read_snapshot().unwrap();

        assert_eq!(snapshot.metrics.len(), 1);
        assert_eq!(*snapshot.metrics.values().next().unwrap(), 12345);
    }
}
