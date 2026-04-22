use std::collections::{HashMap, HashSet};

use joule_profiler_core::sensor::{Sensor, Sensors};
use log::trace;

use crate::{MILLI_JOULE_UNIT, NVML_SOURCE_NAME, Result, error::NvmlError, snapshot::NvmlSnapshot};

/// Trait abstracting NVML hardware access for testability.
pub trait NvmlHardware: Send {
    fn new(gpus_spec: Option<HashSet<u32>>, exit_on_device_failure: bool) -> Result<Self>
    where
        Self: Sized;
    fn read_snapshot(&self) -> Result<NvmlSnapshot>;
    fn get_sensors(&self) -> Result<Sensors>;
}

/// Hardware adapter for NVML library.
pub struct NvmlWrapperHardware {
    /// The NVML wrapper instance for interacting with the NVIDIA driver.
    pub nvml: nvml_wrapper::Nvml,

    /// The indexes of the different GPUs.
    pub devices_indexes: HashSet<u32>,
}

impl NvmlHardware for NvmlWrapperHardware {
    fn new(gpus_spec: Option<HashSet<u32>>, exit_on_device_failure: bool) -> Result<Self> {
        let nvml = nvml_wrapper::Nvml::init().map_err(|err| match err {
            nvml_wrapper::error::NvmlError::DriverNotLoaded => NvmlError::NoDriverLoaded,
            nvml_wrapper::error::NvmlError::NoPermission => NvmlError::NoPermission,
            _ => err.into(),
        })?;

        let devices_max_index = nvml.device_count()?;
        let devices: HashSet<_> = gpus_spec.unwrap_or(
            (0..devices_max_index)
                .collect::<HashSet<_>>()
                .into_iter()
                .collect(),
        );

        let devices_indexes: HashSet<_> = devices
            .into_iter()
            .filter_map(|i| {
                let device = match nvml.device_by_index(i) {
                    Ok(d) => d,
                    Err(e) => return Some(Err(e.into())),
                };

                let name = match device.name() {
                    Ok(n) => n,
                    Err(e) => return Some(Err(e.into())),
                };

                trace!("Discovered NVIDIA device {name}");

                match device.total_energy_consumption() {
                    Ok(_) => {
                        trace!("Added NVIDIA device {name} to devices list");
                        Some(Ok(i))
                    }
                    Err(err) => {
                        if exit_on_device_failure {
                            Some(Err(err.into()))
                        } else {
                            trace!("Skipping NVIDIA device {name}");
                            None
                        }
                    }
                }
            })
            .collect::<Result<_>>()?;

        Ok(NvmlWrapperHardware {
            nvml,
            devices_indexes,
        })
    }

    /// Reads the current energy consumption snapshot for all GPU devices.
    ///
    /// This queries each GPU device and retrieves its total energy consumption counter
    /// value in millijoules.
    fn read_snapshot(&self) -> Result<NvmlSnapshot> {
        let mut gpus_energy = HashMap::with_capacity(self.devices_indexes.len());
        for i in &self.devices_indexes {
            let device = self.nvml.device_by_index(*i)?;
            let energy = device.total_energy_consumption()?;
            gpus_energy.insert(*i, energy);
        }
        Ok(NvmlSnapshot { gpus_energy })
    }

    fn get_sensors(&self) -> Result<Sensors> {
        self.devices_indexes
            .iter()
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
