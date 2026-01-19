use std::{collections::HashMap, ops::AddAssign};

use anyhow::Result;
use log::{debug, error, info, trace};

use crate::{
    core::metric::{Metric, Metrics},
    error::JouleProfilerError,
    sources::rapl::{
        POWERCAP_SOURCE_NAME,
        domain::{RaplDomain, RaplDomainType},
    },
};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Snapshot {
    pub metrics: HashMap<(RaplDomainType, u32), u64>,
}

impl AddAssign<Snapshot> for Snapshot {
    fn add_assign(&mut self, rhs: Snapshot) {
        self.metrics = rhs
            .metrics
            .into_iter()
            .map(|(domain, value)| {
                if let Some(v) = self.metrics.get(&domain) {
                    (domain, value + v)
                } else {
                    (domain, value)
                }
            })
            .collect();
    }
}

impl From<Snapshot> for Metrics {
    fn from(snapshot: Snapshot) -> Self {
        snapshot
            .metrics
            .into_iter()
            .map(|((domain, socket), value)| {
                Metric::new(
                    domain.to_string_socket(socket),
                    value,
                    "uj".to_string(),
                    POWERCAP_SOURCE_NAME.to_lowercase(),
                )
            })
            .collect()
    }
}

impl Snapshot {
    pub fn try_new(metrics: HashMap<(RaplDomainType, u32), u64>) -> Self {
        Self { metrics }
    }
}

/// Compute one measurement from two energy snapshots.
pub fn compute_measurement_from_snapshots(
    domains: &[RaplDomain],
    begin: &Snapshot,
    end: &Snapshot,
) -> Result<HashMap<(RaplDomainType, u32), u64>> {
    trace!(
        "Computing measurement from snapshots for {} domains",
        domains.len()
    );

    let mut per_domain_energy: HashMap<(RaplDomainType, u32), u64> = HashMap::new();

    for domain in domains {
        trace!("Processing domain '{}'", domain.get_name());

        let start_uj = match begin.metrics.get(&(domain.domain_type, domain.socket)) {
            Some(v) => *v,
            None => {
                error!(
                    "Missing start energy snapshot for domain '{}'",
                    domain.get_name()
                );
                return Err(JouleProfilerError::RaplReadError(format!(
                    "Missing start energy snapshot for domain '{}'",
                    domain.get_name()
                ))
                .into());
            }
        };

        let end_uj = match end.metrics.get(&(domain.domain_type, domain.socket)) {
            Some(v) => *v,
            None => {
                error!(
                    "Missing end energy snapshot for domain '{}'",
                    domain.get_name()
                );
                return Err(JouleProfilerError::RaplReadError(format!(
                    "Missing end energy snapshot for domain '{}'",
                    domain.get_name()
                ))
                .into());
            }
        };

        let max_uj = domain.max_energy_uj;
        let diff_uj = energy_diff(start_uj, end_uj, max_uj);
        debug!(
            "Domain '{}': start={} µJ, end={} µJ, diff={} µJ, max={}",
            domain.get_name(),
            start_uj,
            end_uj,
            diff_uj,
            max_uj
        );

        per_domain_energy
            .entry((domain.domain_type, domain.socket))
            .and_modify(|v| *v += diff_uj)
            .or_insert(diff_uj);
    }

    info!(
        "Computed energy measurement for {} domains",
        per_domain_energy.len()
    );

    Ok(per_domain_energy)
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
    use crate::sources::rapl::domain::RaplDomainType;

    use super::*;

    fn snapshot(values: &[(RaplDomainType, u32, u64)]) -> Snapshot {
        Snapshot {
            metrics: values
                .iter()
                .map(|(domain_type, socket, v)| ((*domain_type, *socket), *v))
                .collect(),
        }
    }

    fn domain(
        domain_type: RaplDomainType,
        socket: u32,
        path: &str,
        max_energy_uj: u64,
    ) -> RaplDomain {
        RaplDomain {
            socket,
            domain_type,
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
        let domains = vec![domain(
            RaplDomainType::Package,
            0,
            "/sys/powercap/package0",
            1_000,
        )];

        let begin = snapshot(&[(RaplDomainType::Package, 0, 100)]);
        let end = snapshot(&[(RaplDomainType::Package, 0, 250)]);

        let result = compute_measurement_from_snapshots(&domains, &begin, &end).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[&(RaplDomainType::Package, 0)], 150);
    }

    #[test]
    fn compute_handles_overflow() {
        let domains = vec![domain(
            RaplDomainType::Package,
            0,
            "/sys/powercap/package0",
            1_000,
        )];

        let begin = snapshot(&[(RaplDomainType::Package, 0, 900)]);
        let end = snapshot(&[(RaplDomainType::Package, 0, 100)]);

        let result = compute_measurement_from_snapshots(&domains, &begin, &end).unwrap();

        assert_eq!(result[&(RaplDomainType::Package, 0)], 200);
    }

    #[test]
    fn compute_aggregates_same_domain_same_socket() {
        let domains = vec![
            domain(RaplDomainType::Core, 0, "/core0", 1_000),
            domain(RaplDomainType::Core, 0, "/core1", 1_000),
        ];

        let begin = snapshot(&[
            (RaplDomainType::Core, 0, 100),
            (RaplDomainType::Package, 1, 200),
        ]);
        let end = snapshot(&[
            (RaplDomainType::Core, 0, 300),
            (RaplDomainType::Package, 2, 500),
        ]);

        let result = compute_measurement_from_snapshots(&domains, &begin, &end).unwrap();

        assert_eq!(result[&(RaplDomainType::Core, 0)], 400);
    }

    #[test]
    fn compute_separates_sockets() {
        let domains = vec![
            domain(RaplDomainType::Package, 0, "/pkg0", 1_000),
            domain(RaplDomainType::Package, 1, "/pkg1", 1_000),
        ];

        let begin = snapshot(&[
            (RaplDomainType::Package, 0, 100),
            (RaplDomainType::Package, 1, 400),
        ]);

        let end = snapshot(&[
            (RaplDomainType::Package, 0, 200),
            (RaplDomainType::Package, 1, 700),
        ]);

        let result = compute_measurement_from_snapshots(&domains, &begin, &end).unwrap();

        assert_eq!(result[&(RaplDomainType::Package, 0)], 100);
        assert_eq!(result[&(RaplDomainType::Package, 1)], 300);
    }

    #[test]
    fn error_when_start_snapshot_missing() {
        let domains = vec![domain(RaplDomainType::Package, 0, "/pkg0", 1_000)];

        let begin = snapshot(&[]);
        let end = snapshot(&[(RaplDomainType::Package, 0, 100)]);

        let err = compute_measurement_from_snapshots(&domains, &begin, &end)
            .unwrap_err()
            .to_string();

        assert!(err.contains("Missing start energy snapshot"));
    }

    #[test]
    fn error_when_end_snapshot_missing() {
        let domains = vec![domain(RaplDomainType::Package, 0, "/pkg0", 1_000)];

        let begin = snapshot(&[(RaplDomainType::Package, 0, 100)]);
        let end = snapshot(&[]);

        let err = compute_measurement_from_snapshots(&domains, &begin, &end)
            .unwrap_err()
            .to_string();

        assert!(err.contains("Missing end energy snapshot"));
    }
}
