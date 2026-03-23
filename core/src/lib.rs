#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::struct_field_names
)]

mod aggregate;
pub mod config;
mod orchestrator;
mod phase;
mod profiler;
pub mod sensor;

mod util;
pub use util::fs;

pub mod source;

pub use profiler::{JouleProfiler, JouleProfilerError};

pub mod unit;
pub mod types {
    pub use super::aggregate::{Metric, Metrics, sensor_result::SensorResult};
    pub use super::phase::PhaseToken;
    pub use super::profiler::types::{Iteration, Iterations, Phase, Phases};
}

#[cfg(any(test, feature = "test-utils"))]
pub use source::mock;
