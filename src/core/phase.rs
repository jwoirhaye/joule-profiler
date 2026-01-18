use std::fmt::Display;

use serde::Serialize;

#[derive(Debug, Clone)]
pub enum PhaseToken {
    Start,
    Token(String),
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

/// Detected token with timestamp
#[derive(Debug, Clone)]
pub struct PhaseInfo {
    pub token: PhaseToken,
    pub timestamp: u128,
    pub duration_ms: u128,
    pub line_number: Option<usize>,
}

impl PhaseInfo {
    pub fn new(
        token: PhaseToken,
        timestamp: u128,
        duration_ms: u128,
        line_number: Option<usize>,
    ) -> Self {
        Self {
            token,
            timestamp,
            duration_ms,
            line_number,
        }
    }
}
