use std::collections::HashMap;

use log::{debug, error, info, trace};

use crate::{
    Result, domain_type::RaplDomainIndex, error::RaplError, perf::domain::PerfRaplDomain,
    snapshot::Snapshot,
};

/// Compute one measurement from two energy snapshots.
pub fn compute_measurement_from_snapshots(
    domains: &[PerfRaplDomain],
    begin: &Snapshot,
    end: &Snapshot,
) -> Result<HashMap<RaplDomainIndex, u64>> {
    trace!(
        "Computing measurement from snapshots for {} domains",
        domains.len()
    );

    let mut per_domain_energy: HashMap<RaplDomainIndex, u64> = HashMap::new();

    for domain in domains {
        trace!("Processing domain '{}'", domain.get_name());

        let start_value = match begin.metrics.get(&(domain.domain_type, domain.socket)) {
            Some(v) => *v,
            None => {
                error!(
                    "Missing start energy snapshot for domain '{}'",
                    domain.get_name()
                );
                return Err(RaplError::RaplReadError(format!(
                    "Missing start energy snapshot for domain '{}'",
                    domain.get_name()
                )));
            }
        };

        let end_value = match end.metrics.get(&(domain.domain_type, domain.socket)) {
            Some(v) => *v,
            None => {
                error!(
                    "Missing end energy snapshot for domain '{}'",
                    domain.get_name()
                );
                return Err(RaplError::RaplReadError(format!(
                    "Missing end energy snapshot for domain '{}'",
                    domain.get_name()
                )));
            }
        };

        let diff = energy_diff(start_value, end_value);
        debug!(
            "Domain '{}': start={} µJ, end={} µJ, diff={} µJ",
            domain.get_name(),
            start_value,
            end_value,
            diff,
        );

        per_domain_energy
            .entry((domain.domain_type, domain.socket))
            .and_modify(|v| *v += diff)
            .or_insert(diff);
    }

    info!(
        "Computed energy measurement for {} domains",
        per_domain_energy.len()
    );

    Ok(per_domain_energy)
}

fn energy_diff(start: u64, end: u64) -> u64 {
    if end >= start {
        end - start
    } else {
        (u64::MAX - start) + end
    }
}
