use std::fmt::Display;

use tokio::time::Instant;

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
    pub timestamp: Instant,
    pub line_number: Option<usize>,
}
