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
        (end + max + 1) - start
    }
}
