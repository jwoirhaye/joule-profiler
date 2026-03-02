use std::collections::HashMap;

/// A snapshot of GPU energy consumption.
///
/// This struct holds the total energy consumption (in millijoules) for each GPU device
/// at the time the snapshot was taken.
///
/// # Fields
///
/// * `gpus_energy` - A map from GPU device index to total energy consumption in millijoules.
#[derive(Debug, Default, Clone)]
pub struct NvmlSnapshot {
    pub gpus_energy: HashMap<u32, u64>,
}

#[derive(Debug, Clone, Default)]
pub struct Phase {
    pub begin: NvmlSnapshot,
    pub end: NvmlSnapshot,
}

impl Phase {
    pub fn diff(&self) -> NvmlSnapshot {
        NvmlSnapshot {
            gpus_energy: self
                .end
                .gpus_energy
                .iter()
                .map(|(gpu, end_value)| {
                    let diff = if let Some(begin_value) = self.begin.gpus_energy.get(gpu) {
                        end_value - begin_value
                    } else {
                        0
                    };
                    (*gpu, diff)
                })
                .collect(),
        }
    }
}
