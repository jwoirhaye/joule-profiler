use serde::Serialize;

use crate::core::{aggregate::Metrics, phase::PhaseToken};

/// Represents a profiling phase with metrics and timing
#[derive(Debug, Serialize)]
pub struct Phase {
    /// Token marking the start of the phase
    pub start_token: PhaseToken,

    /// Token marking the end of the phase
    pub end_token: PhaseToken,

    /// Start timestamp in microseconds
    pub timestamp: u128,

    /// Duration of the phase in milliseconds
    pub duration_ms: u128,

    /// Optional line number associated with the phase
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_number: Option<usize>,

    /// Metrics collected during the phase
    pub metrics: Metrics,
}

/// Represents a profiling phase with metrics and timing
impl Phase {
    /// Create a new Phase
    pub fn new(
        metrics: Metrics,
        start_token: PhaseToken,
        end_token: PhaseToken,
        timestamp: u128,
        duration_ms: u128,
        line_number: Option<usize>,
    ) -> Self {
        Self {
            metrics,
            start_token,
            end_token,
            timestamp,
            duration_ms,
            line_number,
        }
    }
}

/// Represents a profiler iteration with its phases and metrics
#[derive(Debug, Serialize)]
pub struct Iteration {
    /// Index of the iteration
    pub index: usize,

    /// Start timestamp in microseconds
    pub timestamp: u128,

    /// Duration of the iteration in milliseconds
    pub duration_ms: u128,

    /// Exit code of the profiled command
    pub exit_code: i32,

    /// Number of measurements performed
    pub measure_count: u64,

    /// Time between measurements in microseconds
    pub measure_delta: u64,

    /// Phases detected in the iteration
    pub phases: Vec<Phase>,
}

impl Iteration {
    /// Create a new Iteration
    pub fn new(
        phases: Vec<Phase>,
        index: usize,
        timestamp: u128,
        duration_ms: u128,
        exit_code: i32,
        measure_count: u64,
        measure_delta: u64,
    ) -> Self {
        Self {
            phases,
            index,
            timestamp,
            duration_ms,
            exit_code,
            measure_count,
            measure_delta,
        }
    }
}
