use std::collections::HashMap;

use crate::rapl::{EnergySnapshot, RaplDomain};
use anyhow::Result;
use log::{debug, error, trace, warn};
use serde::Serialize;

use crate::errors::JouleProfilerError;

#[derive(Debug, Clone, Serialize)]
pub struct MeasurementResult {
    /// Energy per domain (key) in microjoules
    pub energy_uj: HashMap<String, u64>,
    /// Duration in milliseconds
    pub duration_ms: u128,
    /// Command exit code
    pub exit_code: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct PhaseMeasurement {
    pub name: String,
    pub result: MeasurementResult,
}

#[derive(Debug, Clone, Serialize)]
pub struct PhasesResult {
    pub phases: Vec<PhaseMeasurement>,
}

/// Compute one measurement from two energy snapshots.
pub fn compute_measurement_from_snapshots(
    domains: &[&RaplDomain],
    max_map: &HashMap<String, u64>,
    begin: &EnergySnapshot,
    end: &EnergySnapshot,
    duration_ms: u128,
    exit_code: i32,
) -> Result<MeasurementResult> {
    if domains.is_empty() {
        warn!("Computing measurement with no domains");
        return Err(JouleProfilerError::NoDomains.into());
    }

    debug!(
        "Computing measurement from snapshots (duration: {} ms)",
        duration_ms
    );
    trace!(
        "Snapshot timestamps: begin={} µs, end={} µs",
        begin.timestamp_us, end.timestamp_us
    );

    let mut per_domain_socket: HashMap<(String, u32), u64> = HashMap::new();

    for d in domains {
        let key = d.path.to_string_lossy().to_string();
        trace!(
            "Processing domain '{}' (socket {}) at {:?}",
            d.name, d.socket, d.path
        );

        let start_uj = begin.energies_uj.get(&key).copied().ok_or_else(|| {
            error!(
                "Missing start energy for domain '{}' (key: {})",
                d.name, key
            );
            JouleProfilerError::RaplReadError(format!(
                "Missing start energy snapshot for domain '{}'",
                d.name
            ))
        })?;

        let end_uj = end.energies_uj.get(&key).copied().ok_or_else(|| {
            error!("Missing end energy for domain '{}' (key: {})", d.name, key);
            JouleProfilerError::RaplReadError(format!(
                "Missing end energy snapshot for domain '{}'",
                d.name
            ))
        })?;

        let max_uj = max_map.get(&key).copied();

        trace!(
            "Domain '{}': start={} µJ, end={} µJ, max={:?} µJ",
            d.name, start_uj, end_uj, max_uj
        );

        let diff_uj = energy_diff(start_uj, end_uj, max_uj, &d.name)?;

        if diff_uj == 0 && start_uj != end_uj {
            warn!(
                "Zero energy difference for domain '{}' despite different values (start={}, end={}, no max available)",
                d.name, start_uj, end_uj
            );
        }

        trace!("Domain '{}' energy consumption: {} µJ", d.name, diff_uj);

        per_domain_socket
            .entry((d.name.clone(), d.socket))
            .and_modify(|v| {
                trace!(
                    "Accumulating {} µJ to existing {} µJ for {}",
                    diff_uj, *v, d.name
                );
                *v += diff_uj
            })
            .or_insert(diff_uj);
    }

    debug!(
        "Processed {} domain(s), building final energy map",
        per_domain_socket.len()
    );

    let mut energy_uj: HashMap<String, u64> = HashMap::new();

    debug!("Keeping per-socket energy breakdown");
    for ((name, socket), val_uj) in per_domain_socket {
        let key = format!("{}_{}", name.to_uppercase(), socket);
        trace!("Recording {} µJ for domain-socket key '{}'", val_uj, key);
        energy_uj.insert(key, val_uj);
    }

    debug!(
        "Measurement computed: {} domain key(s), total duration {} ms",
        energy_uj.len(),
        duration_ms
    );

    if !energy_uj.is_empty() {
        let total_energy: u64 = energy_uj.values().sum();
        debug!(
            "Total energy consumption: {} µJ ({:.3} J)",
            total_energy,
            total_energy as f64 / 1_000_000.0
        );
        for (key, value) in &energy_uj {
            trace!(
                "  {}: {} µJ ({:.3} J)",
                key,
                value,
                *value as f64 / 1_000_000.0
            );
        }
    }

    Ok(MeasurementResult {
        energy_uj,
        duration_ms,
        exit_code,
    })
}

pub fn energy_diff(start: u64, end: u64, max: Option<u64>, domain_name: &str) -> Result<u64> {
    if end >= start {
        let diff = end - start;
        trace!("Energy diff (normal): {} - {} = {} µJ", end, start, diff);
        Ok(diff)
    } else if let Some(max) = max {
        // Counter wrapped around
        let diff = end + max - start;
        warn!(
            "Energy counter wrapped for domain '{}': end={} < start={}, using max={} → diff={} µJ",
            domain_name, end, start, max, diff
        );

        if diff > max / 2 {
            error!(
                "Suspicious overflow for domain '{}': calculated diff ({} µJ) is > 50% of max range ({} µJ)",
                domain_name, diff, max
            );
            return Err(JouleProfilerError::CounterOverflow.into());
        }

        Ok(diff)
    } else {
        error!(
            "Counter overflow without max_energy for domain '{}': end={} < start={}, cannot compute accurate measurement",
            domain_name, end, start
        );
        Err(JouleProfilerError::CounterOverflow.into())
    }
}

pub fn build_max_map(domains: &[&RaplDomain]) -> HashMap<String, u64> {
    debug!("Building max_energy map for {} domain(s)", domains.len());
    let mut m = HashMap::new();
    let mut count_with_max = 0;

    for d in domains {
        if let Some(max) = d.max_energy_uj {
            let key = d.path.to_string_lossy().to_string();
            trace!(
                "Domain '{}' (socket {}): max_energy = {} µJ",
                d.name, d.socket, max
            );
            m.insert(key, max);
            count_with_max += 1;
        } else {
            trace!(
                "Domain '{}' (socket {}): no max_energy_range_uj available",
                d.name, d.socket
            );
        }
    }

    debug!(
        "Max energy map built: {}/{} domain(s) have max_energy configured",
        count_with_max,
        domains.len()
    );

    m
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_energy_diff_normal() {
        let result = energy_diff(1000, 1500, Some(10000), "test").unwrap();
        assert_eq!(result, 500);
    }

    #[test]
    fn test_energy_diff_overflow_with_max() {
        let result = energy_diff(9500, 500, Some(10000), "test").unwrap();
        assert_eq!(result, 1000);
    }

    #[test]
    fn test_energy_diff_overflow_without_max() {
        let result = energy_diff(9500, 500, None, "test");
        assert!(result.is_err());

        if let Err(e) = result {
            let err = e.downcast::<JouleProfilerError>().unwrap();
            assert!(matches!(err, JouleProfilerError::CounterOverflow));
        }
    }

    #[test]
    fn test_energy_diff_suspicious_overflow() {
        let result = energy_diff(8000, 4000, Some(10000), "test");
        assert!(result.is_err(), "Expected error for suspicious overflow");

        if let Err(e) = result {
            let err = e.downcast::<JouleProfilerError>().unwrap();
            assert!(
                matches!(err, JouleProfilerError::CounterOverflow),
                "Expected CounterOverflow error"
            );
        }
    }

    #[test]
    fn test_energy_diff_zero() {
        let result = energy_diff(1000, 1000, Some(10000), "test").unwrap();
        assert_eq!(result, 0);
    }

    #[test]
    fn test_energy_diff_edge_case_max_boundary() {
        let result = energy_diff(9999, 0, Some(10000), "test").unwrap();
        assert_eq!(result, 1);
    }

    #[test]
    fn test_energy_diff_overflow_exactly_50_percent() {
        let result = energy_diff(5000, 0, Some(10000), "test").unwrap();
        assert_eq!(result, 5000);
    }

    #[test]
    fn test_energy_diff_overflow_just_over_50_percent() {
        let result = energy_diff(4999, 0, Some(10000), "test");
        assert!(result.is_err(), "Expected error for overflow just over 50%");

        if let Err(e) = result {
            let err = e.downcast::<JouleProfilerError>().unwrap();
            assert!(matches!(err, JouleProfilerError::CounterOverflow));
        }
    }

    #[test]
    fn test_energy_diff_large_overflow() {
        let result = energy_diff(9900, 9000, Some(10000), "test");
        assert!(result.is_err(), "Expected error for large overflow");

        if let Err(e) = result {
            let err = e.downcast::<JouleProfilerError>().unwrap();
            assert!(matches!(err, JouleProfilerError::CounterOverflow));
        }
    }
}
