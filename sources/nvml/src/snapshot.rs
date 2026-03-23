use std::collections::HashMap;

/// A snapshot of GPU energy consumption.
///
/// This struct holds the total energy consumption (in millijoules) for each GPU device
/// at the time the snapshot was taken.
#[derive(Debug, Default, Clone)]
pub struct NvmlSnapshot {
    /// A map from GPU device index to total energy consumption in millijoules.
    pub gpus_energy: HashMap<u32, u64>,
}

#[derive(Debug, Clone, Default)]
pub struct Phase {
    /// The begin snapshot made at the start of a phase.
    pub begin: NvmlSnapshot,

    /// The snapshot made at the end of a phase.
    pub end: NvmlSnapshot,
}

impl Phase {
    /// Compute the differences between the start and the end of a snapshot.
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
