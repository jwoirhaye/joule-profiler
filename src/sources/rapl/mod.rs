use std::{
    collections::{HashMap, HashSet},
    env,
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
        domain::{RaplDomain, get_domains, read_energy},
        snapshot::{Snapshot, compute_measurement_from_snapshots},
    },
    util::platform::check_os,
};

pub mod domain;
pub mod snapshot;

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

        let rapl = Rapl::try_new(config.rapl_path.as_deref(), sockets, rapl_polling)?;
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

    fn init(&mut self) -> Result<()> {
        let ticker = if let Some(duration) = self.poll_interval {
            let timerfd_interval = Interval::new_interval(duration)?;
            Some(timerfd_interval)
        } else {
            None
        };
        self.ticker = ticker;
        Ok(())
    }

    fn measure(&self) -> Result<Self::Type> {
        trace!("Starting RAPL measurement");
        Ok(self.read_snapshot()?)
    }

    fn compute_measures(&self, new: &Self::Type, old: Self::Type) -> Result<Self::Type> {
        let metrics = compute_measurement_from_snapshots(&self.domains, &old, new)?;
        let snapshot = Self::Type::try_new(metrics);
        Ok(snapshot)
    }

    async fn poll(&mut self) -> Option<Result<()>> {
        if let Some(ticker) = &mut self.ticker {
            let _ = ticker.next().await?;
            Some(Ok(()))
        } else {
            None
        }
    }
}

impl Rapl {
    pub fn try_new(
        rapl_path: Option<&str>,
        sockets: Option<&HashSet<u32>>,
        polling_rate_s: Option<f64>,
    ) -> Result<Self> {
        check_os()?;

        let base_path = rapl_base_path(rapl_path);
        check_rapl(&base_path)?;

        let domains = get_domains(&base_path, sockets)?;
        let poll_interval = polling_rate_s.map(Duration::from_secs_f64);

        Ok(Rapl {
            domains,
            poll_interval,
            ticker: None,
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

/// Checks if the RAPL interface is available at the given base path.
fn check_rapl(base: &str) -> Result<()> {
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

/// Resolves the RAPL base path from configuration and environment.
pub fn rapl_base_path(config_override: Option<&str>) -> String {
    if let Some(path) = config_override {
        return path.to_string();
    }

    if let Ok(env_path) = env::var("JOULE_PROFILER_RAPL_PATH") {
        return env_path;
    }

    let default_path = "/sys/devices/virtual/powercap/intel-rapl";
    default_path.to_string()
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
