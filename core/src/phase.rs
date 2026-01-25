use std::fmt::Display;

use serde::Serialize;

/// Represents a phase marker in an iteration
#[derive(Debug, Clone, PartialEq)]
pub enum PhaseToken {
    /// Start of a phase
    Start,

    /// Custom token detected in output
    Token(String),

    /// End of a phase
    End,
}

impl Serialize for PhaseToken {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
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

/// Detected phase with timestamp and optional line number
#[derive(Debug, Clone)]
pub struct PhaseInfo {
    /// Phase token detected
    pub token: PhaseToken,

    /// Timestamp in milliseconds
    pub timestamp: u128,

    // /// Duration of the phase in milliseconds
    // pub duration_ms: u128,
    /// Optional line number in output where token was detected
    pub line_number: Option<usize>,
}

impl PhaseInfo {
    /// Create a new PhaseInfo
    pub fn new(
        token: PhaseToken,
        timestamp: u128,
        // duration_ms: u128,
        line_number: Option<usize>,
    ) -> Self {
        Self {
            token,
            timestamp,
            // duration_ms,
            line_number,
        }
    }
}
