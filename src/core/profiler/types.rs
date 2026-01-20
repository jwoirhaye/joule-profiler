use serde::Serialize;

use crate::core::{metric::Metrics, phase::PhaseToken};

#[derive(Debug, Serialize)]
pub struct Phase {
    pub start_token: PhaseToken,

    pub end_token: PhaseToken,

    pub timestamp: u128,

    pub duration_ms: u128,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_number: Option<usize>,

    pub metrics: Metrics,
}

impl Phase {
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

#[derive(Debug, Serialize)]
pub struct Iteration {
    pub index: usize,
    pub timestamp: u128,
    pub duration_ms: u128,
    pub exit_code: i32,
    pub measure_count: u64,
    pub measure_delta: u64,
    pub phases: Vec<Phase>,
}

impl Iteration {
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
