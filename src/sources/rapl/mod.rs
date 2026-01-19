use std::{
    collections::{HashMap, HashSet},
    env, fs,
    io::ErrorKind,
    path::Path,
    time::Duration,
};

use futures::{FutureExt, StreamExt};
use log::{debug, error, info, trace};
use tokio_timerfd::Interval;

use crate::{
    config::{Command, Config},
    core::{
        sensor::{Sensor, Sensors},
        source::MetricReaderTrait,
    },
    sources::rapl::{
        domain::{RaplDomain, get_domains, read_energy},
        error::RaplError,
        snapshot::{Snapshot, compute_measurement_from_snapshots},
    },
};

mod domain;
pub mod error;
pub mod snapshot;

const POWERCAP_SOURCE_NAME: &str = "Powercap";

pub type Result<T> = std::result::Result<T, RaplError>;

#[derive(Default)]
pub struct Rapl {
    domains: Vec<RaplDomain>,
    ticker: Option<Interval>,
}

impl TryFrom<&Config> for Rapl {
    type Error = RaplError;

    fn try_from(config: &Config) -> Result<Self> {
        let base_path = rapl_base_path(config.rapl_path.as_deref());

        let (sockets, rapl_polling) = match &config.mode {
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

impl MetricReaderTrait for Rapl {
    type Type = Snapshot;
    type Error = RaplError;

    fn measure(&self) -> Result<Self::Type> {
        trace!("Starting RAPL measurement");
        self.read_snapshot()
    }

    fn compute_measures(&self, new: &Self::Type, old: Self::Type) -> Result<Self::Type> {
        let metrics = compute_measurement_from_snapshots(&self.domains, &old, new)?;
        let snapshot = Self::Type::try_new(metrics);
        Ok(snapshot)
    }

    async fn poll(&mut self) -> Option<Result<()>> {
        if let Some(ticker) = &mut self.ticker {
            let _ = ticker.next().await?;
            ticker
                .next()
                .map(|option| option.map(|result| result.map_err(RaplError::IoError)))
                .await
        } else {
            None
        }
    }

    fn get_sensors(&self) -> Result<Sensors> {
        let sensors = self
            .domains
            .iter()
            .map(|domain| {
                Sensor::new(
                    domain.get_name(),
                    "µJ".to_string(),
                    POWERCAP_SOURCE_NAME.to_lowercase(),
                )
            })
            .collect();

        Ok(sensors)
    }
}

impl Rapl {
    pub fn try_new(
        rapl_path: &str,
        sockets: Option<&HashSet<u32>>,
        polling_rate_s: Option<f64>,
    ) -> Result<Self> {
        check_os()?;
        check_rapl(rapl_path)?;

        let domains = get_domains(rapl_path, sockets)?;
        let poll_interval = polling_rate_s.map(Duration::from_secs_f64);

        let ticker = if let Some(duration) = poll_interval {
            let timerfd_interval = Interval::new_interval(duration)?;
            Some(timerfd_interval)
        } else {
            None
        };

        let rapl = Rapl::new(domains, ticker);
        Ok(rapl)
    }

    pub fn read_snapshot(&self) -> Result<Snapshot> {
        trace!(
            "Reading energy snapshot from {} domains",
            self.domains.len()
        );

        let mut map = HashMap::with_capacity(self.domains.len());

        for domain in &self.domains {
            let val_uj = read_energy(domain)?;
            map.insert((domain.domain_type, domain.socket), val_uj);
        }

        Ok(Snapshot { metrics: map })
    }

    fn new(domains: Vec<RaplDomain>, ticker: Option<Interval>) -> Self {
        Self { domains, ticker }
    }
}

/// Checks if the RAPL interface is available at the given base path.
fn check_rapl(base: &str) -> Result<()> {
    debug!("Checking RAPL base path: {}", base);

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

/// Checks if the operating system is Linux.
pub fn check_os() -> Result<()> {
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
