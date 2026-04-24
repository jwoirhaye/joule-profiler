use std::fmt::Display;

use serde::Serialize;

/// Represents a phase marker, indicating the beginning or the end of a phase.
#[derive(Debug, Clone, PartialEq)]
pub enum PhaseToken {
    /// Start phase.
    Start,

    /// Custom token detected in standard output, marking the beginning of a phase.
    Token(String),

    /// Ending phase.
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

/// Detected phase with timestamp and optional line number.
#[derive(Debug, Clone)]
pub struct PhaseInfo {
    /// Phase token detected.
    pub token: PhaseToken,

    /// Timestamp in microseconds.
    pub timestamp: u128,

    /// Optional line number in output where token was detected.
    pub line_number: Option<usize>,
}

impl PhaseInfo {
    pub fn start(timestamp: u128) -> Self {
        Self {
            token: PhaseToken::Start,
            timestamp,
            line_number: None,
        }
    }

    pub fn end(timestamp: u128) -> Self {
        Self {
            token: PhaseToken::End,
            timestamp,
            line_number: None,
        }
    }
}
