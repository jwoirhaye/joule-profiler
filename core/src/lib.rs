mod aggregate;
pub mod config;
mod orchestrator;
mod phase;
mod profiler;
pub mod sensor;

mod util;
pub use util::{fs, sys};

pub mod source;

pub use profiler::{JouleProfiler, JouleProfilerError};

pub mod unit;
pub mod types {
    pub use super::aggregate::{Metric, Metrics, sensor_result::SensorResult};
    pub use super::profiler::types::{Iteration, Iterations, Phase, Phases};
}
