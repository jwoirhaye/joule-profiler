use std::ops::{Add, AddAssign};

use crate::core::aggregate::phase::SensorPhase;

#[derive(Default, Debug)]
pub struct SensorIteration {
    pub phases: Vec<SensorPhase>,
    pub measure_delta: u64,
    pub measure_count: u64,
}

impl SensorIteration {
    pub fn new(phases: Vec<SensorPhase>, measure_delta: u64, measure_count: u64) -> Self {
        Self {
            phases,
            measure_delta,
            measure_count,
        }
    }
}

impl AddAssign for SensorIteration {
    fn add_assign(&mut self, rhs: Self) {
        self.phases
            .iter_mut()
            .zip(rhs.phases)
            .for_each(|(self_phase, rhs_phase)| *self_phase += rhs_phase);
    }
}

impl Add for SensorIteration {
    type Output = SensorIteration;

    fn add(mut self, rhs: Self) -> Self::Output {
        self += rhs;
        self
    }
}
