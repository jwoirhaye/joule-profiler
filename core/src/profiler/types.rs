use crate::aggregate::Metrics;
use crate::phase::PhaseToken;
use serde::Serialize;

/// Represents a profiling phase with metrics and timing
#[derive(Debug, Serialize)]
pub struct Phase {
    /// The index of the phase
    pub index: usize,

    /// Token marking the start of the phase
    pub start_token: PhaseToken,

    /// Token marking the end of the phase
    pub end_token: PhaseToken,

    /// Start timestamp in milliseconds
    pub timestamp: u128,

    /// Duration of the phase in milliseconds
    pub duration_ms: u128,

    /// Optional start line number associated with the phase
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_line: Option<usize>,

    /// Optional end line number associated with the phase
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_line: Option<usize>,

    /// Metrics collected during the phase
    pub metrics: Metrics,
}

/// Represents a profiling phase with metrics and timing
impl Phase {
    pub fn get_name(&self) -> String {
        format!("{} -> {}", self.start_token, self.end_token)
    }
}

pub type Phases = Vec<Phase>;

/// Represents a profiler iteration with its phases and metrics
#[derive(Debug, Serialize)]
pub struct Iteration {
    /// Index of the iteration
    pub index: usize,

    /// Start timestamp in milliseconds
    pub timestamp: u128,

    /// Duration of the iteration in milliseconds
    pub duration_ms: u128,

    /// Exit code of the profiled command
    pub exit_code: i32,

    /// Phases detected in the iteration
    pub phases: Phases,
}

impl Iteration {
    /// Create a new Iteration
    pub fn new(
        phases: Phases,
        index: usize,
        timestamp: u128,
        duration_ms: u128,
        exit_code: i32,
    ) -> Self {
        Self {
            phases,
            index,
            timestamp,
            duration_ms,
            exit_code,
        }
    }
}

pub type Iterations = Vec<Iteration>;
