pub mod domain;
pub mod snapshot;
pub mod measure;

use anyhow::{Result, bail};
pub use domain::{
    RaplDomain, check_os, check_rapl, discover_domains, discover_sockets, parse_sockets,
};
pub use snapshot::{EnergySnapshot, read_snapshot};

use log::{debug, info, trace};
use std::env;

use crate::{errors::JouleProfilerError, source::{MetricReader, Metrics, metric::Metric, rapl::measure::compute_measurement_from_snapshots}};

pub struct Rapl {
    domains: Vec<RaplDomain>,
    measures: Vec<EnergySnapshot>,
}

impl MetricReader for Rapl {
    fn measure(&mut self) -> Result<()> {
        self.measures.push(read_snapshot(&self.domains)?);
        Ok(())
    }

    fn retrieve(&mut self) -> Result<Vec<Metrics>> {
        if self.measures.len() < 2 {
            bail!(JouleProfilerError::NotEnoughSnapshots)
        }

        let mut snapshots_metrics = Vec::with_capacity(self.measures.len() - 1);

        for i in 1..self.measures.len() {
            let begin_measure = &self.measures[i - 1];
            let end_measure = &self.measures[i];
            let measure = compute_measurement_from_snapshots(&self.domains, begin_measure, end_measure)?;

            let total_uj: u64 = measure.values().sum();
            
            let mut metrics: Metrics = measure.into_iter().map(|(key, value)| Metric::new(key, value, "powercap".to_string(), "uj".to_string())).collect();
            metrics.push(Metric { name: "total_uj".to_string(), value: total_uj, source: "powercap".to_string(), unit: "uj".to_string() });
            
            snapshots_metrics.push(metrics);
        }

        Ok(snapshots_metrics)
    }
}

impl Rapl {
    pub fn new(domains: Vec<RaplDomain>) -> Result<Self> {
        Ok(Self { domains, measures: Vec::default() })
    }
}

pub fn get_domains(base_path: Option<&str>, spec: Option<&str>) -> Result<Vec<RaplDomain>> {
    check_os()?;

    let base = rapl_base_path(base_path);
    check_rapl(&base)?;

    let domains = discover_domains(&base)?;
    let sockets = parse_or_all_sockets(spec, &domains)?;

    let filtered: Vec<RaplDomain> = domains
        .into_iter()
        .filter(|d| sockets.contains(&d.socket))
        .collect();

    Ok(filtered)
}

/// Resolves the RAPL base path from configuration and environment.
pub fn rapl_base_path(config_override: Option<&str>) -> String {
    trace!("Resolving RAPL base path");

    if let Some(path) = config_override {
        info!("Using RAPL path from CLI override: {}", path);
        return path.to_string();
    }

    if let Ok(env_path) = env::var("JOULE_PROFILER_RAPL_PATH") {
        info!(
            "Using RAPL path from environment variable JOULE_PROFILER_RAPL_PATH: {}",
            env_path
        );
        return env_path;
    }

    let default_path = "/sys/devices/virtual/powercap/intel-rapl";
    debug!("Using default RAPL path: {}", default_path);
    default_path.to_string()
}

pub fn parse_or_all_sockets(spec: Option<&str>, domains: &[RaplDomain]) -> Result<Vec<u32>> {
    if let Some(spec) = spec {
        debug!("Parsing socket specification: {}", spec);
        let sockets = crate::source::rapl::parse_sockets(spec, domains)?;
        info!("Using specified sockets: {:?}", sockets);
        Ok(sockets)
    } else {
        let sockets = discover_sockets(domains);
        debug!(
            "No socket specification, using all discovered sockets: {:?}",
            sockets
        );
        Ok(sockets)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_rapl_base_path_default() {
        unsafe {
            env::remove_var("JOULE_PROFILER_RAPL_PATH");
        }

        let path = rapl_base_path(None);
        assert_eq!(path, "/sys/devices/virtual/powercap/intel-rapl");
    }

    #[test]
    #[serial]
    fn test_rapl_base_path_cli_override() {
        unsafe {
            env::remove_var("JOULE_PROFILER_RAPL_PATH");
        }

        let custom_path = "/custom/rapl/path".to_string();
        let path = rapl_base_path(Some(&custom_path));
        assert_eq!(path, "/custom/rapl/path");
    }

    #[test]
    #[serial]
    fn test_rapl_base_path_env_variable() {
        let env_path = "/env/rapl/path";
        unsafe {
            env::set_var("JOULE_PROFILER_RAPL_PATH", env_path);
        }

        let path = rapl_base_path(None);
        assert_eq!(path, env_path);

        unsafe {
            env::remove_var("JOULE_PROFILER_RAPL_PATH");
        }
    }

    #[test]
    #[serial]
    fn test_rapl_base_path_cli_takes_precedence_over_env() {
        let env_path = "/env/rapl/path";
        let cli_path = "/cli/rapl/path".to_string();

        unsafe {
            env::set_var("JOULE_PROFILER_RAPL_PATH", env_path);
        }

        let path = rapl_base_path(Some(&cli_path));
        assert_eq!(path, "/cli/rapl/path");

        unsafe {
            env::remove_var("JOULE_PROFILER_RAPL_PATH");
        }
    }

    #[test]
    #[serial]
    fn test_rapl_base_path_env_takes_precedence_over_default() {
        let env_path = "/env/rapl/path";
        unsafe {
            env::set_var("JOULE_PROFILER_RAPL_PATH", env_path);
        }

        let path = rapl_base_path(None);
        assert_eq!(path, env_path);
        assert_ne!(path, "/sys/devices/virtual/powercap/intel-rapl");

        unsafe {
            env::remove_var("JOULE_PROFILER_RAPL_PATH");
        }
    }

    #[test]
    #[serial]
    fn test_rapl_base_path_empty_string_override() {
        unsafe {
            env::remove_var("JOULE_PROFILER_RAPL_PATH");
        }

        let empty_path = "".to_string();
        let path = rapl_base_path(Some(&empty_path));
        assert_eq!(path, "");
    }

    #[test]
    #[serial]
    fn test_rapl_base_path_with_trailing_slash() {
        unsafe {
            env::remove_var("JOULE_PROFILER_RAPL_PATH");
        }

        let path_with_slash = "/custom/rapl/path/".to_string();
        let path = rapl_base_path(Some(&path_with_slash));
        assert_eq!(path, "/custom/rapl/path/");
    }

    #[test]
    #[serial]
    fn test_rapl_base_path_relative_path() {
        unsafe {
            env::remove_var("JOULE_PROFILER_RAPL_PATH");
        }

        let relative = "./relative/rapl".to_string();
        let path = rapl_base_path(Some(&relative));
        assert_eq!(path, "./relative/rapl");
    }
}
