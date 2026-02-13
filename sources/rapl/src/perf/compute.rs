use std::collections::HashMap;

use log::{debug, error, info, trace};

use crate::{
    Result, domain_type::RaplDomainIndex, error::RaplError, perf::socket::Socket,
    snapshot::Snapshot,
};

/// Compute one measurement from two energy snapshots.
pub fn compute_measurement_from_snapshots(
    sockets: &[Socket],
    begin: &Snapshot,
    end: &Snapshot,
) -> Result<HashMap<RaplDomainIndex, u64>> {
    trace!(
        "Computing measurement from snapshots for {} sockets",
        sockets.len()
    );

    let mut per_domain_energy: HashMap<RaplDomainIndex, u64> = HashMap::new();

    for socket in sockets {
        trace!(
            "Computing measurement from snapshots for {} domains for socket {}",
            socket.domains.len(),
            socket.id
        );

        for domain in &socket.domains {
            let domain_name = domain.get_name(socket.id);
            trace!("Processing domain '{}'", domain_name);

            let domain_index = (domain.domain_type, socket.id);

            let start_value = match begin.metrics.get(&domain_index) {
                Some(v) => *v,
                None => {
                    error!("Missing start energy snapshot for domain '{}'", domain_name);
                    return Err(RaplError::RaplReadError(format!(
                        "Missing start energy snapshot for domain '{}'",
                        domain_name
                    )));
                }
            };

            let end_value = match end.metrics.get(&domain_index) {
                Some(v) => *v,
                None => {
                    error!("Missing end energy snapshot for domain '{}'", domain_name);
                    return Err(RaplError::RaplReadError(format!(
                        "Missing end energy snapshot for domain '{}'",
                        domain_name
                    )));
                }
            };

            let diff = energy_diff(start_value, end_value);
            debug!(
                "Domain '{}': start={} µJ, end={} µJ, diff={} µJ",
                domain_name, start_value, end_value, diff,
            );

            per_domain_energy
                .entry(domain_index)
                .and_modify(|v| *v += diff)
                .or_insert(diff);
        }
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

/// Convert joules to microjoules.
#[inline]
pub fn joules_to_micro_joules(joules: f64) -> u64 {
    (joules * 1_000_000.0) as u64
}
