use std::fmt::Display;

use serde::Serialize;

use crate::source::Metric;

#[derive(Debug, Clone)]
pub enum PhaseToken {
    Start,
    Token(String),
    End,
}

impl Display for PhaseToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PhaseToken::Start => f.write_str("START"),
            PhaseToken::Token(token) => f.write_str(token),
            PhaseToken::End => f.write_str("END"),
        }
    }
}

impl From<PhaseToken> for Option<String> {
    fn from(token: PhaseToken) -> Self {
        match token {
            PhaseToken::Start | PhaseToken::End => None,
            PhaseToken::Token(token) => Some(token),
        }
    }
}

/// Detected token with timestamp
#[derive(Debug, Clone)]
pub struct Phase {
    pub token: PhaseToken,
    pub timestamp: u128,
    pub line_number: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PhaseResult {
    pub name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_token: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_token: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_line: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_line: Option<usize>,

    pub metrics: Vec<Metric>,

    pub duration_ms: u128,
}

impl PhaseResult {
    pub fn new(
        start_phase_token: &PhaseToken,
        end_phase_token: &PhaseToken,
        start_line: Option<usize>,
        end_line: Option<usize>,
        metrics: Vec<Metric>,
        duration_ms: u128,
    ) -> Self {
        let name = format!("{} -> {}", start_phase_token, end_phase_token,);
        Self {
            name,
            start_token: start_phase_token.clone().into(),
            end_token: end_phase_token.clone().into(),
            duration_ms,
            start_line,
            end_line,
            metrics,
        }
    }

    pub fn extract_keys(&self) -> Vec<&String> {
        self.metrics.iter().map(|metric| &metric.name).collect()
    }
}

pub struct PhaseMeasurementResult {
    /// The metrics of each phase
    pub phases: Vec<PhaseResult>,
    /// Duration in milliseconds
    pub duration_ms: u128,
    /// Command exit code
    pub exit_code: i32,
}

impl PhaseMeasurementResult {
    pub fn extract_keys(&self) -> Vec<&String> {
        self.phases
            .iter()
            .flat_map(|phase| phase.extract_keys())
            .collect()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct MeasurementResult {
    /// Metrics measured
    pub metrics: Vec<Metric>,
    /// Duration in milliseconds
    pub duration_ms: u128,
    /// Command exit code
    pub exit_code: i32,
    /// The number of measures made by the sources
    pub measure_count: u64,

    pub measure_delta: u128,
}

impl MeasurementResult {
    pub fn extract_keys(&self) -> Vec<&String> {
        self.metrics.iter().map(|metric| &metric.name).collect()
    }
}
