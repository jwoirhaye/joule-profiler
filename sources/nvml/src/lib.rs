//! NVML (NVIDIA Management Library) energy profiling integration.
//!
//! This module provides energy consumption monitoring for NVIDIA GPUs using the NVML library.
//! It implements the `MetricReader` trait to collect energy metrics from GPU devices and
//! track energy usage over time.

use std::collections::HashMap;

use joule_profiler_core::{
    sensor::{Sensor, Sensors},
    source::MetricReader,
    types::{Metric, Metrics},
    unit::{MetricUnit, Unit, UnitPrefix},
};
use log::{debug, trace};

use crate::{
    error::NvmlError,
    snapshot::{NvmlSnapshot, Phase},
};

mod error;
mod snapshot;

const NVML_SOURCE_NAME: &str = "NVML";
const MILLI_JOULE_UNIT: MetricUnit = MetricUnit {
    prefix: UnitPrefix::Milli,
    unit: Unit::Joule,
};

/// Custom result type for NVML.
type Result<T> = std::result::Result<T, NvmlError>;

/// NVML-based energy profiler for NVIDIA GPUs.
///
/// This struct provides an interface to monitor energy consumption of NVIDIA GPUs using
/// the NVML library.
#[derive(Debug)]
pub struct Nvml {
    /// The NVML wrapper instance for interacting with the NVIDIA driver.
    nvml: nvml_wrapper::Nvml,

    /// The total number of GPU devices detected.
    devices_max_index: u32,

    /// Accumulated energy consumption since last retrieval.
    begin_snapshot: Option<NvmlSnapshot>,

    /// The most recent snapshot taken, used to compute deltas.
    last_snapshot: Option<NvmlSnapshot>,
}

impl Nvml {
    /// Creates a new NVML profiler instance.
    ///
    /// This initializes the NVML library and queries all available GPU devices.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The NVML library cannot be initialized (driver not installed, incompatible version, etc.)
    /// - Device information cannot be queried.
    /// - No GPU devices are detected.
    /// - The permissions are insufficient to be able to query the NVML driver.
    pub fn new() -> Result<Self> {
        debug!("Attempting to initialize NVML reader");
        let nvml = nvml_wrapper::Nvml::init().map_err(|err| match err {
            nvml_wrapper::error::NvmlError::DriverNotLoaded => NvmlError::NoDriverLoaded,
            nvml_wrapper::error::NvmlError::NoPermission => NvmlError::NoPermission,
            _ => err.into(),
        })?;
        let devices_max_index = nvml.device_count()?;
        for i in 0..devices_max_index {
            let device = nvml.device_by_index(i)?;
            trace!("Discovered NVIDIA device {}", device.name()?);
        }

        Ok(Self {
            nvml,
            devices_max_index,
            begin_snapshot: None,
            last_snapshot: None,
        })
    }

    /// Reads the current energy consumption snapshot for all GPU devices.
    ///
    /// This queries each GPU device and retrieves its total energy consumption counter
    /// value in millijoules.
    fn read_snapshot(&self) -> Result<NvmlSnapshot> {
        let mut gpus_energy = HashMap::with_capacity(self.devices_max_index as usize);
        for i in 0..self.devices_max_index {
            let device = self.nvml.device_by_index(i)?;
            let energy = device.total_energy_consumption()?;
            gpus_energy.insert(i, energy);
        }
        Ok(NvmlSnapshot { gpus_energy })
    }

    /// Computes the energy consumption difference between two snapshots.
    ///
    /// This calculates the delta in energy consumption for each GPU device between
    /// the old and new snapshots. Uses wrapping subtraction to handle counter wraparound
    /// correctly (even so it will never occur in theory).
    fn compute_energy_diff(
        end_snapshot: &NvmlSnapshot,
        begin_snapshot: &NvmlSnapshot,
    ) -> Result<NvmlSnapshot> {
        let mut gpus_energy = HashMap::new();
        for (device_index, old_energy) in &begin_snapshot.gpus_energy {
            let new_energy =
                end_snapshot
                    .gpus_energy
                    .get(device_index)
                    .ok_or(NvmlError::UnknownMetricError(format!(
                        "Device {} unknown",
                        device_index
                    )))?;
            let diff = new_energy.wrapping_sub(*old_energy);
            gpus_energy.insert(*device_index, diff);
        }
        Ok(NvmlSnapshot { gpus_energy })
    }
}

impl MetricReader for Nvml {
    type Type = Phase;

    type Error = NvmlError;

    async fn measure(&mut self) -> Result<()> {
        let new_snapshot = self.read_snapshot()?;
        if self.begin_snapshot.is_none() {
            self.begin_snapshot = Some(new_snapshot);
        } else {
            self.last_snapshot = Some(new_snapshot);
        }
        Ok(())
    }

    async fn retrieve(&mut self) -> Result<Self::Type> {
        if let Some(begin) = self.begin_snapshot.take()
            && let Some(end) = self.last_snapshot.take()
        {
            self.begin_snapshot = Some(end.clone());
            Ok(Phase { begin, end })
        } else {
            Err(NvmlError::NotEnoughSamples)
        }
    }

    fn get_sensors(&self) -> Result<Sensors> {
        (0..self.devices_max_index)
            .map(|i| {
                Ok(Sensor {
                    name: format!("GPU-{}", i),
                    unit: MILLI_JOULE_UNIT,
                    source: NVML_SOURCE_NAME.to_string(),
                })
            })
            .collect::<Result<_>>()
    }

    fn get_name() -> &'static str {
        NVML_SOURCE_NAME
    }

    fn to_metrics(&self, result: Self::Type) -> Result<Metrics> {
        let diff = Self::compute_energy_diff(&result.end, &result.begin)?;
        Ok(diff
            .gpus_energy
            .into_iter()
            .map(|(device_index, energy)| Metric {
                name: format!("GPU-{}", device_index),
                value: energy,
                unit: MILLI_JOULE_UNIT,
                source: NVML_SOURCE_NAME.to_string(),
            })
            .collect())
    }

    async fn reset(&mut self) -> Result<()> {
        self.begin_snapshot = None;
        self.last_snapshot = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::snapshot::NvmlSnapshot;

    fn snapshot(entries: Vec<(u32, u64)>) -> NvmlSnapshot {
        NvmlSnapshot {
            gpus_energy: entries.into_iter().collect(),
        }
    }

    #[test]
    fn diff_basic() {
        let begin = snapshot(vec![(0, 100), (1, 200)]);
        let end = snapshot(vec![(0, 150), (1, 300)]);
        let diff = Nvml::compute_energy_diff(&end, &begin).unwrap();
        assert_eq!(diff.gpus_energy[&0], 50);
        assert_eq!(diff.gpus_energy[&1], 100);
    }

    #[test]
    fn diff_zero_when_equal() {
        let snap = snapshot(vec![(0, 500)]);
        let diff = Nvml::compute_energy_diff(&snap, &snap).unwrap();
        assert_eq!(diff.gpus_energy[&0], 0);
    }

    #[test]
    fn diff_wraps_on_counter_overflow() {
        let begin = snapshot(vec![(0, u64::MAX - 5)]);
        let end = snapshot(vec![(0, 10)]);
        let diff = Nvml::compute_energy_diff(&end, &begin).unwrap();
        assert_eq!(diff.gpus_energy[&0], 16);
    }

    #[test]
    fn diff_multiple_devices() {
        let begin = snapshot(vec![(0, 0), (1, 1000), (2, 500)]);
        let end = snapshot(vec![(0, 42), (1, 1200), (2, 500)]);
        let diff = Nvml::compute_energy_diff(&end, &begin).unwrap();
        assert_eq!(diff.gpus_energy[&0], 42);
        assert_eq!(diff.gpus_energy[&1], 200);
        assert_eq!(diff.gpus_energy[&2], 0);
    }

    #[test]
    fn diff_device_missing_in_end_returns_error() {
        let begin = snapshot(vec![(0, 100), (1, 200)]);
        let end = snapshot(vec![(0, 150)]); // device 1 missing
        let result = Nvml::compute_energy_diff(&end, &begin);
        assert!(matches!(result, Err(NvmlError::UnknownMetricError(_))));
    }

    #[test]
    fn diff_empty_snapshots_returns_empty() {
        let diff = Nvml::compute_energy_diff(&snapshot(vec![]), &snapshot(vec![])).unwrap();
        assert!(diff.gpus_energy.is_empty());
    }

    #[test]
    fn to_metrics_produces_one_metric_per_gpu() {
        let begin = snapshot(vec![(0, 0), (1, 0)]);
        let end = snapshot(vec![(0, 100), (1, 200)]);
        let diff = Nvml::compute_energy_diff(&end, &begin).unwrap();

        let mut metrics: Vec<Metric> = diff
            .gpus_energy
            .into_iter()
            .map(|(i, energy)| Metric {
                name: format!("GPU-{}", i),
                value: energy,
                unit: MILLI_JOULE_UNIT,
                source: NVML_SOURCE_NAME.to_string(),
            })
            .collect();

        metrics.sort_by_key(|m| m.name.clone());

        assert_eq!(metrics.len(), 2);
        assert_eq!(metrics[0].name, "GPU-0");
        assert_eq!(metrics[0].value, 100);
        assert_eq!(metrics[0].unit, MILLI_JOULE_UNIT);
        assert_eq!(metrics[1].name, "GPU-1");
        assert_eq!(metrics[1].value, 200);
    }
}
