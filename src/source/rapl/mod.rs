pub mod domain;
pub mod measure;
pub mod snapshot;

use anyhow::{Result, bail};
pub use domain::{
    RaplDomain, check_os, check_rapl, discover_domains, discover_sockets, parse_sockets,
};
use serde::Serialize;
pub use snapshot::{EnergySnapshot, read_snapshot};

use log::{debug, info, trace};
use std::env;

use crate::{
    config::OutputFormat,
    errors::JouleProfilerError,
    source::{
        MetricReader,
        metric::{Metric, Snapshot},
        rapl::measure::compute_measurement_from_snapshots,
    },
};

#[derive(Clone)]
pub struct Rapl {
    pub domains: Vec<RaplDomain>,
    measures: Vec<EnergySnapshot>,
}

impl MetricReader for Rapl {
    fn measure(&mut self) -> Result<()> {
        self.measures.push(read_snapshot(&self.domains)?);
        Ok(())
    }

    fn retrieve(&mut self) -> Result<Vec<Snapshot>> {
        if self.measures.len() < 2 {
            bail!(JouleProfilerError::NotEnoughSnapshots)
        }

        let measures = std::mem::take(&mut self.measures);
        let mut snapshots_metrics = Vec::with_capacity(measures.len() - 1);

        for window in measures.windows(2) {
            let (begin_measure, end_measure) = (&window[0], &window[1]);
            let measure =
                compute_measurement_from_snapshots(&self.domains, begin_measure, end_measure)?;

            let total_uj: u64 = measure.values().sum();

            let mut metrics: Vec<Metric> = measure
                .into_iter()
                .map(|(key, value)| {
                    Metric::new(key, value, "powercap".to_string(), "µj".to_string())
                })
                .collect();
            metrics.sort_by_key(|metric| metric.name.clone());
            metrics.push(Metric {
                name: "total_energy".to_string(),
                value: total_uj,
                source: "powercap".to_string(),
                unit: "µj".to_string(),
            });

            let snapshot = Snapshot {
                metrics,
                timestamp_us: begin_measure.timestamp_us,
            };

            snapshots_metrics.push(snapshot);
        }

        Ok(snapshots_metrics)
    }

    fn print_source(&self, format: OutputFormat) -> Result<()> {
        let mut infos: Vec<DomainInfo> = self
            .domains
            .iter()
            .map(|d| DomainInfo {
                socket: d.socket,
                name: d.name.to_uppercase(),
                raw_name: d.name.clone(),
                path: d.path.to_string_lossy().into(),
            })
            .collect();

        infos.sort_by(|a, b| a.socket.cmp(&b.socket).then_with(|| a.name.cmp(&b.name)));

        match format {
            OutputFormat::Json => print_domains_json(&infos),
            OutputFormat::Csv => print_domains_csv(&infos),
            OutputFormat::Terminal => print_domains_table(&infos),
        }
    }
}

impl Rapl {
    pub fn new(domains: Vec<RaplDomain>) -> Result<Self> {
        Ok(Self {
            domains,
            measures: Vec::default(),
        })
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

#[derive(Debug, Serialize)]
pub struct DomainInfo {
    socket: u32,
    name: String,
    raw_name: String,
    path: String,
}

fn print_domains_table(infos: &[DomainInfo]) -> Result<()> {
    if infos.is_empty() {
        println!("No RAPL domains found for the selected sockets.");
        return Ok(());
    }

    println!();
    println!("Available RAPL domains:");

    let mut current_socket: Option<u32> = None;

    for info in infos {
        if current_socket != Some(info.socket) {
            current_socket = Some(info.socket);
            println!("\nSocket {}:\n", info.socket);
            println!("  {:<16} {:<20} PATH", "NAME", "RAW_NAME",);
            println!("  {:<16} {:<20} ----", "----", "--------");
        }

        println!("  {:<16} {:<20} {}", info.name, info.raw_name, info.path);
    }

    println!();
    Ok(())
}

fn print_domains_json(infos: &[DomainInfo]) -> Result<()> {
    let value = serde_json::to_value(infos)?;
    println!("{}", serde_json::to_string_pretty(&value)?);
    Ok(())
}

fn print_domains_csv(infos: &[DomainInfo]) -> Result<()> {
    use std::io::{self, Write};

    let mut out = io::stdout();

    writeln!(out, "socket;name;raw_name;path")?;

    for info in infos {
        writeln!(
            out,
            "{};{};{};{}",
            info.socket, info.name, info.raw_name, info.path
        )?;
    }

    Ok(())
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
