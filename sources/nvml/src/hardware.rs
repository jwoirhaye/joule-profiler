use std::collections::HashMap;

use joule_profiler_core::sensor::{Sensor, Sensors};
use log::trace;

use crate::{MILLI_JOULE_UNIT, NVML_SOURCE_NAME, Result, error::NvmlError, snapshot::NvmlSnapshot};

/// Trait abstracting NVML hardware access for testability.
pub trait NvmlHardware: Send {
    fn new() -> Result<Self>
    where
        Self: Sized;
    fn read_snapshot(&self) -> Result<NvmlSnapshot>;
    fn get_sensors(&self) -> Result<Sensors>;
}

/// Hardware adapter for NVML library.
pub struct NvmlWrapperHardware {
    /// The NVML wrapper instance for interacting with the NVIDIA driver.
    pub nvml: nvml_wrapper::Nvml,

    /// The total number of GPU devices detected.
    pub devices_max_index: u32,
}

impl NvmlHardware for NvmlWrapperHardware {
    fn new() -> Result<Self> {
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

        Ok(NvmlWrapperHardware {
            nvml,
            devices_max_index,
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

    fn get_sensors(&self) -> Result<Sensors> {
        (0..self.devices_max_index)
            .map(|i| {
                Ok(Sensor {
                    name: format!("GPU-{i}"),
                    unit: MILLI_JOULE_UNIT,
                    source: NVML_SOURCE_NAME.to_string(),
                })
            })
            .collect::<Result<_>>()
    }
}
