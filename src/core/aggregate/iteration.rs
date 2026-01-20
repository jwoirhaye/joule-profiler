use std::ops::{Add, AddAssign};

use crate::core::aggregate::phase::SensorPhase;

/// A single sensor measurement iteration
#[derive(Default, Debug)]
pub struct SensorIteration {
    /// Measured phases for this iteration
    pub phases: Vec<SensorPhase>,

    /// Interval between measurements
    pub measure_delta: u64,

    /// Number of collected measurements
    pub measure_count: u64,
}

impl SensorIteration {
    /// Creates a new sensor iteration
    pub fn new(phases: Vec<SensorPhase>, measure_delta: u64, measure_count: u64) -> Self {
        Self {
            phases,
            measure_delta,
            measure_count,
        }
    }
}

impl AddAssign for SensorIteration {
    /// Aggregates another iteration into this one
    fn add_assign(&mut self, rhs: Self) {
        self.phases
            .iter_mut()
            .zip(rhs.phases)
            .for_each(|(self_phase, rhs_phase)| *self_phase += rhs_phase);
    }
}

impl Add for SensorIteration {
    type Output = SensorIteration;

    /// Returns the sum of two sensor iterations
    fn add(mut self, rhs: Self) -> Self::Output {
        self += rhs;
        self
    }
}
