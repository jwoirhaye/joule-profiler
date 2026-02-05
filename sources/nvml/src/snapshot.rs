use std::{collections::HashMap, ops::AddAssign};

/// A snapshot of GPU energy consumption at a specific point in time.
///
/// This struct holds the total energy consumption (in millijoules) for each GPU device
/// at the time the snapshot was taken. The energy values are cumulative since the GPU
/// was powered on or reset.
///
/// # Fields
///
/// * `gpus_energy` - A map from GPU device index to total energy consumption in millijoules.
#[derive(Debug, Default)]
pub struct NvmlSnapshot {
    pub gpus_energy: HashMap<u32, u64>,
}

impl AddAssign for NvmlSnapshot {
    fn add_assign(&mut self, rhs: Self) {
        for (gpu_name, energy) in rhs.gpus_energy {
            self.gpus_energy
                .entry(gpu_name)
                .or_default()
                .add_assign(energy);
        }
    }
}