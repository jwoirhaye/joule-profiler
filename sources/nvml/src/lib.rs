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
