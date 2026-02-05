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
    unit::{MetricPrefix, MetricUnit, Unit},
};
use log::{debug, trace};

use crate::{error::NvmlError, snapshot::NvmlSnapshot};

mod error;
mod snapshot;

const NVML_SOURCE_NAME: &str = "NVML";
const MILLI_JOULE_UNIT: MetricUnit = MetricUnit {
    prefix: MetricPrefix::Milli,
    unit: Unit::Joule,
};

/// Custom result type for NVML
type Result<T> = std::result::Result<T, NvmlError>;

/// NVML-based energy profiler for NVIDIA GPUs.
///
/// This struct provides an interface to monitor energy consumption of NVIDIA GPUs using
/// the NVML library.
///
/// # Fields
///
/// * `nvml` - The NVML wrapper instance for interacting with the NVIDIA driver.
/// * `device_names` - Names of all detected GPU devices.
/// * `devices_max_index` - The total number of GPU devices detected.
/// * `current_counters` - Accumulated energy consumption since last retrieval.
/// * `last_snapshot` - The most recent snapshot taken, used to compute deltas.
#[derive(Debug)]
pub struct Nvml {
    nvml: nvml_wrapper::Nvml,

    devices_max_index: u32,

    current_counters: NvmlSnapshot,

    last_snapshot: Option<NvmlSnapshot>,
}

impl Nvml {
    /// Creates a new NVML profiler instance.
    ///
    /// This initializes the NVML library and queries all available GPU devices.
    /// The device names are cached for use in metric reporting.
    ///
    /// # Returns
    ///
    /// * `Ok(Self)` - A new NVML profiler instance with all devices initialized.
    /// * `Err(NvmlError)` - If NVML initialization fails or device information cannot be retrieved.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// * The NVML library cannot be initialized (driver not installed, incompatible version, etc.)
    /// * Device information cannot be queried
    /// * No GPU devices are detected
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
            current_counters: Default::default(),
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
        &self,
        new_snapshot: &NvmlSnapshot,
        old_snapshot: NvmlSnapshot,
    ) -> Result<NvmlSnapshot> {
        let mut gpus_energy = HashMap::new();
        for (device_index, old_energy) in old_snapshot.gpus_energy {
            let new_energy = new_snapshot.gpus_energy.get(&device_index).ok_or(
                NvmlError::UnknownMetricError(format!("Device {} unknown", device_index)),
            )?;
            let diff = new_energy.wrapping_sub(old_energy);
            gpus_energy.insert(device_index, diff);
        }
        Ok(NvmlSnapshot { gpus_energy })
    }
}

impl MetricReader for Nvml {
    type Type = NvmlSnapshot;

    type Error = NvmlError;

    /// Make a measure and accumulate current counters.
    async fn measure(&mut self) -> Result<()> {
        let new_snapshot = self.read_snapshot()?;
        if let Some(last_snapshot) = self.last_snapshot.take() {
            trace!("Computing delta from previous snapshot");
            self.current_counters += self.compute_energy_diff(&new_snapshot, last_snapshot)?;
        }
        self.last_snapshot = Some(new_snapshot);
        Ok(())
    }

    /// Retrieve current counter and reset it
    async fn retrieve(&mut self) -> Result<Self::Type> {
        let counters = std::mem::take(&mut self.current_counters);
        Ok(counters)
    }

    /// Get the sensors by iterating over the detected devices.
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

    /// Get the NVML metric source name.
    fn get_name() -> &'static str {
        NVML_SOURCE_NAME
    }

    /// Convert an NvmlSnapshot into Metrics.
    fn to_metrics(&self, result: Self::Type) -> Metrics {
        result
            .gpus_energy
            .into_iter()
            .map(|(device_index, energy)| Metric {
                name: format!("GPU-{}", device_index),
                value: energy,
                unit: MILLI_JOULE_UNIT.to_string(),
                source: NVML_SOURCE_NAME.to_string(),
            })
            .collect()
    }
}
