use std::collections::HashMap;

use anyhow::Result;
use log::{debug, error, info, trace};

use crate::{error::JouleProfilerError, source::rapl::domain::RaplDomain};

#[derive(Debug, Clone)]
pub struct EnergySnapshot {
    pub energies_uj: HashMap<String, u64>,
    pub timestamp_us: u128,
}

/// Compute one measurement from two energy snapshots.
pub fn compute_measurement_from_snapshots(
    domains: &[RaplDomain],
    begin: &EnergySnapshot,
    end: &EnergySnapshot,
) -> Result<HashMap<String, u64>> {
    trace!(
        "Computing measurement from snapshots for {} domains",
        domains.len()
    );

    let mut per_domain_socket: HashMap<(String, u32), u64> = HashMap::new();

    for domain in domains {
        let key = domain.path.to_string_lossy().to_string();
        trace!("Processing domain '{}'", domain.name);

        let start_uj = match begin.energies_uj.get(&key) {
            Some(v) => *v,
            None => {
                error!("Missing start energy snapshot for domain '{}'", domain.name);
                return Err(JouleProfilerError::RaplReadError(format!(
                    "Missing start energy snapshot for domain '{}'",
                    domain.name
                ))
                .into());
            }
        };

        let end_uj = match end.energies_uj.get(&key) {
            Some(v) => *v,
            None => {
                error!("Missing end energy snapshot for domain '{}'", domain.name);
                return Err(JouleProfilerError::RaplReadError(format!(
                    "Missing end energy snapshot for domain '{}'",
                    domain.name
                ))
                .into());
            }
        };

        let max_uj = domain.max_energy_uj;
        let diff_uj = energy_diff(start_uj, end_uj, max_uj);
        debug!(
            "Domain '{}', socket {}: start={} µJ, end={} µJ, diff={} µJ, max={}",
            domain.name, domain.socket, start_uj, end_uj, diff_uj, max_uj
        );

        per_domain_socket
            .entry((domain.name.clone(), domain.socket))
            .and_modify(|v| *v += diff_uj)
            .or_insert(diff_uj);
    }

    let mut energy_uj: HashMap<String, u64> = HashMap::new();

    for ((name, socket), val_uj) in per_domain_socket {
        let key = format!("{}_{}", name.to_uppercase(), socket);
        energy_uj.insert(key.clone(), val_uj);
        trace!("Final energy for {} = {} µJ", key, val_uj);
    }

    info!(
        "Computed energy measurement for {} domains",
        energy_uj.len()
    );

    Ok(energy_uj)
}

/// Compute the energy difference between two measures, handle overflows with max value.
fn energy_diff(start: u64, end: u64, max: u64) -> u64 {
    if end >= start {
        end - start
    } else {
        (max - start) + end
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snapshot(values: &[(&str, u64)]) -> EnergySnapshot {
        EnergySnapshot {
            energies_uj: values.iter().map(|(k, v)| (k.to_string(), *v)).collect(),
            timestamp_us: 0,
        }
    }

    fn domain(name: &str, socket: u32, path: &str, max_energy_uj: u64) -> RaplDomain {
        RaplDomain {
            name: name.to_string(),
            socket,
            path: path.into(),
            max_energy_uj,
        }
    }

    #[test]
    fn energy_diff_without_overflow() {
        let diff = super::energy_diff(100, 250, 1_000);
        assert_eq!(diff, 150);
    }

    #[test]
    fn energy_diff_with_overflow() {
        let diff = super::energy_diff(900, 100, 1_000);
        assert_eq!(diff, 200);
    }

    #[test]
    fn energy_diff_exact_wrap() {
        let diff = super::energy_diff(900, 0, 1_000);
        assert_eq!(diff, 100);
    }

    #[test]
    fn compute_single_domain_single_socket() {
        let domains = vec![domain("package", 0, "/sys/powercap/package0", 1_000)];

        let begin = snapshot(&[("/sys/powercap/package0", 100)]);
        let end = snapshot(&[("/sys/powercap/package0", 250)]);

        let result = compute_measurement_from_snapshots(&domains, &begin, &end).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result["PACKAGE_0"], 150);
    }

    #[test]
    fn compute_handles_overflow() {
        let domains = vec![domain("package", 0, "/sys/powercap/package0", 1_000)];

        let begin = snapshot(&[("/sys/powercap/package0", 900)]);
        let end = snapshot(&[("/sys/powercap/package0", 100)]);

        let result = compute_measurement_from_snapshots(&domains, &begin, &end).unwrap();

        assert_eq!(result["PACKAGE_0"], 200);
    }

    #[test]
    fn compute_aggregates_same_domain_same_socket() {
        let domains = vec![
            domain("core", 0, "/core0", 1_000),
            domain("core", 0, "/core1", 1_000),
        ];

        let begin = snapshot(&[("/core0", 100), ("/core1", 200)]);

        let end = snapshot(&[("/core0", 300), ("/core1", 500)]);

        let result = compute_measurement_from_snapshots(&domains, &begin, &end).unwrap();

        // (300-100) + (500-200) = 200 + 300 = 500
        assert_eq!(result["CORE_0"], 500);
    }

    #[test]
    fn compute_separates_sockets() {
        let domains = vec![
            domain("package", 0, "/pkg0", 1_000),
            domain("package", 1, "/pkg1", 1_000),
        ];

        let begin = snapshot(&[("/pkg0", 100), ("/pkg1", 400)]);

        let end = snapshot(&[("/pkg0", 200), ("/pkg1", 700)]);

        let result = compute_measurement_from_snapshots(&domains, &begin, &end).unwrap();

        assert_eq!(result["PACKAGE_0"], 100);
        assert_eq!(result["PACKAGE_1"], 300);
    }

    #[test]
    fn error_when_start_snapshot_missing() {
        let domains = vec![domain("package", 0, "/pkg0", 1_000)];

        let begin = snapshot(&[]);
        let end = snapshot(&[("/pkg0", 100)]);

        let err = compute_measurement_from_snapshots(&domains, &begin, &end)
            .unwrap_err()
            .to_string();

        assert!(err.contains("Missing start energy snapshot"));
    }

    #[test]
    fn error_when_end_snapshot_missing() {
        let domains = vec![domain("package", 0, "/pkg0", 1_000)];

        let begin = snapshot(&[("/pkg0", 100)]);
        let end = snapshot(&[]);

        let err = compute_measurement_from_snapshots(&domains, &begin, &end)
            .unwrap_err()
            .to_string();

        assert!(err.contains("Missing end energy snapshot"));
    }
}
