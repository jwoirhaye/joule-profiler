use std::collections::HashMap;
use std::fs;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::Result;
use log::{debug, error, info, trace, warn};

use super::RaplDomain;
use crate::errors::JouleProfilerError;

#[derive(Debug, Clone)]
pub struct EnergySnapshot {
    pub energies_uj: HashMap<String, u64>, // key: path as string
    pub timestamp_us: u128,
}

impl EnergySnapshot {
    pub fn diff(&self, before: &EnergySnapshot) -> Result<HashMap<String, u64>> {
        let mut diff = HashMap::new();

        for (path, after_energy) in &self.energies_uj {
            if let Some(&before_energy) = before.energies_uj.get(path) {
                let delta = if after_energy >= &before_energy {
                    after_energy - before_energy
                } else {
                    warn!(
                        "Counter overflow detected for {}: before={}, after={}",
                        path, before_energy, after_energy
                    );
                    (u64::MAX - before_energy) + after_energy
                };
                diff.insert(path.clone(), delta);
                trace!("Energy delta for {}: {} µJ", path, delta);
            } else {
                warn!("Domain {} not found in previous snapshot", path);
            }
        }

        Ok(diff)
    }

    pub fn duration_us(&self, before: &EnergySnapshot) -> u128 {
        self.timestamp_us.saturating_sub(before.timestamp_us)
    }
}

pub fn read_snapshot(domains: &[&RaplDomain]) -> Result<EnergySnapshot> {
    if domains.is_empty() {
        warn!("Attempting to read snapshot with no domains");
        return Err(JouleProfilerError::NoDomains.into());
    }

    trace!(
        "Starting energy snapshot reading for {} domain(s): [{}]",
        domains.len(),
        domains
            .iter()
            .map(|d| &d.name)
            .cloned()
            .collect::<Vec<_>>()
            .join(", ")
    );

    let mut map = HashMap::with_capacity(domains.len());

    for d in domains {
        trace!(
            "Reading energy from domain: {} (socket {}) at {:?}",
            d.name, d.socket, d.path
        );

        let val_str = fs::read_to_string(&d.path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                error!(
                    "Permission denied reading energy from domain '{}' at {:?}",
                    d.name, d.path
                );
                JouleProfilerError::InsufficientPermissions
            } else {
                error!(
                    "Failed to read energy_uj from domain '{}' at {:?}: {}",
                    d.name, d.path, e
                );
                JouleProfilerError::RaplReadError(format!(
                    "Failed to read energy from domain '{}': {}",
                    d.name, e
                ))
            }
        })?;

        let val_uj: u64 = val_str.trim().parse().map_err(|e| {
            error!(
                "Failed to parse energy value from domain '{}' at {:?}: '{}' (error: {})",
                d.name,
                d.path,
                val_str.trim(),
                e
            );
            JouleProfilerError::ParseEnergyError(format!(
                "Invalid energy value '{}' in domain '{}': {}",
                val_str.trim(),
                d.name,
                e
            ))
        })?;

        trace!(
            "Domain '{}' (socket {}): {} µJ ({:.6} J)",
            d.name,
            d.socket,
            val_uj,
            val_uj as f64 / 1_000_000.0
        );

        if let Some(max_energy) = d.max_energy_uj {
            let usage_percent = (val_uj as f64 / max_energy as f64) * 100.0;
            if usage_percent > 90.0 {
                warn!(
                    "Domain '{}' energy counter at {:.1}% of max range ({} / {} µJ) - overflow risk",
                    d.name, usage_percent, val_uj, max_energy
                );
            }
        }

        map.insert(d.path.to_string_lossy().to_string(), val_uj);
    }

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|e| {
            warn!("System time is before UNIX_EPOCH: {}, using 0", e);
            Duration::from_secs(0)
        });
    let timestamp_us = now.as_micros();

    debug!(
        "✓ Energy snapshot completed: {} domain(s) read at timestamp {} µs ({} ms since epoch)",
        map.len(),
        timestamp_us,
        timestamp_us / 1000
    );

    Ok(EnergySnapshot {
        energies_uj: map,
        timestamp_us,
    })
}

pub fn compute_energy_diff(
    after: &EnergySnapshot,
    before: &EnergySnapshot,
    domains: &[&RaplDomain],
) -> Result<HashMap<String, u64>> {
    trace!("Computing energy difference between snapshots");

    let duration_us = after.duration_us(before);
    let duration_ms = duration_us as f64 / 1000.0;

    debug!(
        "Snapshot time difference: {} µs ({:.3} ms)",
        duration_us, duration_ms
    );

    let mut result = HashMap::with_capacity(domains.len());

    for d in domains {
        let path_str = d.path.to_string_lossy().to_string();

        let before_val = before.energies_uj.get(&path_str).copied().ok_or_else(|| {
            error!("Domain '{}' not found in before snapshot", d.name);
            JouleProfilerError::RaplReadError(format!(
                "Domain '{}' missing in before snapshot",
                d.name
            ))
        })?;

        let after_val = after.energies_uj.get(&path_str).copied().ok_or_else(|| {
            error!("Domain '{}' not found in after snapshot", d.name);
            JouleProfilerError::RaplReadError(format!(
                "Domain '{}' missing in after snapshot",
                d.name
            ))
        })?;

        let delta = if after_val >= before_val {
            after_val - before_val
        } else {
            warn!(
                "Counter overflow detected for domain '{}': before={} µJ, after={} µJ",
                d.name, before_val, after_val
            );

            if let Some(max_energy) = d.max_energy_uj {
                let overflow_delta = (max_energy - before_val) + after_val;
                warn!(
                    "Calculated overflow delta for '{}': {} µJ (using max_energy_range_uj: {} µJ)",
                    d.name, overflow_delta, max_energy
                );
                overflow_delta
            } else {
                warn!(
                    "No max_energy_range_uj available for '{}', assuming 64-bit counter",
                    d.name
                );
                (u64::MAX - before_val) + after_val
            }
        };

        let power_w = if duration_us > 0 {
            (delta as f64 / 1_000_000.0) / (duration_us as f64 / 1_000_000.0)
        } else {
            0.0
        };

        trace!(
            "Domain '{}': Δ{} µJ ({:.6} J), avg power: {:.3} W",
            d.name,
            delta,
            delta as f64 / 1_000_000.0,
            power_w
        );

        result.insert(d.name.clone(), delta);
    }

    let total_energy_uj: u64 = result.values().sum();
    let total_energy_j = total_energy_uj as f64 / 1_000_000.0;
    let avg_power_w = if duration_us > 0 {
        total_energy_j / (duration_us as f64 / 1_000_000.0)
    } else {
        0.0
    };

    info!(
        "Energy diff computed: {} domain(s), total: {:.6} J ({} µJ), avg power: {:.3} W over {:.3} ms",
        result.len(),
        total_energy_j,
        total_energy_uj,
        avg_power_w,
        duration_ms
    );

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_diff_normal() {
        let mut before_map = HashMap::new();
        before_map.insert("domain1".to_string(), 1000);
        before_map.insert("domain2".to_string(), 2000);

        let before = EnergySnapshot {
            energies_uj: before_map,
            timestamp_us: 1000000,
        };

        let mut after_map = HashMap::new();
        after_map.insert("domain1".to_string(), 1500);
        after_map.insert("domain2".to_string(), 2300);

        let after = EnergySnapshot {
            energies_uj: after_map,
            timestamp_us: 1100000,
        };

        let diff = after.diff(&before).unwrap();
        assert_eq!(diff.get("domain1"), Some(&500));
        assert_eq!(diff.get("domain2"), Some(&300));
        assert_eq!(after.duration_us(&before), 100000);
    }

    #[test]
    fn test_snapshot_diff_overflow() {
        let mut before_map = HashMap::new();
        before_map.insert("domain1".to_string(), u64::MAX - 100);

        let before = EnergySnapshot {
            energies_uj: before_map,
            timestamp_us: 1000000,
        };

        let mut after_map = HashMap::new();
        after_map.insert("domain1".to_string(), 200);

        let after = EnergySnapshot {
            energies_uj: after_map,
            timestamp_us: 1100000,
        };

        let diff = after.diff(&before).unwrap();
        assert_eq!(diff.get("domain1"), Some(&300));
    }
}
