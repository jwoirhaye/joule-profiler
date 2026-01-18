use std::collections::HashSet;

use super::RaplDomain;

/// Discovers all unique socket indices from the given RAPL domains.
pub fn discover_sockets(domains: &[RaplDomain]) -> HashSet<u32> {
    domains.iter().map(|d| d.socket).collect()
}

/// Parses a socket specification string and validates against available domains.
pub fn filter_sockets(spec: &HashSet<u32>, sockets: &HashSet<u32>) -> HashSet<u32> {
    spec.intersection(sockets).cloned().collect()
}

/// Filter RAPL sockets with specified spec.
pub fn parse_or_all_sockets(domains: &[RaplDomain], spec: Option<&HashSet<u32>>) -> Vec<u32> {
    let mut sockets = discover_sockets(domains);
    if let Some(spec) = spec {
        sockets = filter_sockets(spec, &sockets);
    }
    sockets.into_iter().collect()
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use crate::sources::rapl::domain::{
        RaplDomain, RaplDomainType,
        socket::{discover_sockets, filter_sockets, parse_or_all_sockets},
    };

    #[test]
    fn discover_sockets_returns_unique_sockets() {
        let domains = vec![
            RaplDomain {
                path: "a".into(),
                domain_type: RaplDomainType::Package,
                socket: 0,
                max_energy_uj: 1,
            },
            RaplDomain {
                path: "b".into(),
                domain_type: RaplDomainType::Package,
                socket: 1,
                max_energy_uj: 1,
            },
        ];

        let sockets = discover_sockets(&domains);
        assert_eq!(sockets.len(), 2);
        assert!(sockets.contains(&0));
        assert!(sockets.contains(&1));
    }

    #[test]
    fn filter_sockets_intersects_correctly() {
        let available: HashSet<u32> = [0, 1, 2].into_iter().collect();
        let spec: HashSet<u32> = [1, 3].into_iter().collect();

        let filtered = filter_sockets(&spec, &available);

        assert_eq!(filtered.len(), 1);
        assert!(filtered.contains(&1));
    }

    #[test]
    fn parse_or_all_sockets_without_spec_returns_all() {
        let domains = vec![
            RaplDomain {
                path: "a".into(),
                domain_type: RaplDomainType::Package,
                socket: 0,
                max_energy_uj: 1,
            },
            RaplDomain {
                path: "b".into(),
                domain_type: RaplDomainType::Package,
                socket: 1,
                max_energy_uj: 1,
            },
        ];

        let sockets = parse_or_all_sockets(&domains, None);
        assert_eq!(sockets.len(), 2);
    }
}
