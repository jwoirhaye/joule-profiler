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
    /// The snapshot made at the start of a phase.
    pub begin: NvmlSnapshot,

    /// The snapshot made at the end of a phase.
    pub end: NvmlSnapshot,
}
