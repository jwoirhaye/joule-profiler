use std::{collections::HashMap, ops::AddAssign};

use crate::domain_type::RaplDomainIndex;

/// Snapshot of energy counters
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Snapshot {
    pub metrics: HashMap<RaplDomainIndex, u64>,
}

impl AddAssign<HashMap<RaplDomainIndex, u64>> for Snapshot {
    fn add_assign(&mut self, rhs: HashMap<RaplDomainIndex, u64>) {
        for (domain, value) in rhs {
            *self.metrics.entry(domain).or_default() += value;
        }
    }
}

/// A pair of snapshots delimiting a phase.
#[derive(Debug, Clone, Default)]
pub struct Phase {
    /// The snapshot made at the start of a phase.
    pub begin: Snapshot,

    /// End snapshot of the phase.
    pub end: Snapshot,
}
