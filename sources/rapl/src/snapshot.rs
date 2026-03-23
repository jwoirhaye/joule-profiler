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

#[derive(Debug, Clone, Default)]
pub struct Phase {
    pub begin: Snapshot,
    pub end: Snapshot,
}

impl Phase {
    /// Computes the difference between to snapshots.
    pub fn diff(&self) -> Snapshot {
        Snapshot {
            metrics: self
                .end
                .metrics
                .iter()
                .map(|(domain, end_value)| {
                    let diff = if let Some(begin_value) = self.begin.metrics.get(domain) {
                        end_value - begin_value
                    } else {
                        0
                    };
                    (*domain, diff)
                })
                .collect(),
        }
    }
}
