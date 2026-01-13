use thiserror::Error;

#[derive(Debug, Error)]
pub enum JouleProfilerError {
    #[error("Intel RAPL not available at {0}")]
    RaplNotAvailable(String),

    #[error("No RAPL domains found")]
    NoDomains,

    #[error("Failed to read RAPL domain: {0}")]
    RaplReadError(String),

    #[error("Invalid RAPL domain path: {0}")]
    InvalidRaplPath(String),

    #[error("RAPL counter overflow detected")]
    CounterOverflow,

    #[error("No command specified")]
    NoCommand,

    #[error("Invalid iterations value: {0}. Must be >= 1")]
    InvalidIterations(usize),

    #[error("Cannot use both --json and --csv flags simultaneously")]
    ConflictingOutputFormats,

    #[error("Invalid socket specification: {0}")]
    InvalidSocketSpec(String),

    #[error("Socket {0} not found in available RAPL domains")]
    SocketNotFound(u32),

    #[error("Failed to execute command: {0}")]
    CommandExecutionFailed(String),

    #[error("Command not found: {0}")]
    CommandNotFound(String),

    #[error("Command killed by signal: {0}")]
    CommandKilled(i32),

    #[error("Token '{0}' not found in program output")]
    TokenNotFound(String),

    #[error("End token '{end}' found before start token '{start}'")]
    InvalidTokenOrder { start: String, end: String },

    #[error("Multiple occurrences of token '{0}' found (expected exactly one)")]
    MultipleTokens(String),

    #[error("Failed to create output file: {0}")]
    OutputFileCreationFailed(String),

    #[error("Failed to write output: {0}")]
    OutputWriteFailed(String),

    #[error("Output format not supported for this mode")]
    UnsupportedOutputFormat,

    #[error("Unsupported operating system: {0}. Only Linux is supported")]
    UnsupportedOS(String),

    #[error("Insufficient permissions to access RAPL. Try running with sudo")]
    InsufficientPermissions,

    #[error("Failed to get current directory")]
    CurrentDirNotFound,

    #[error("Failed to parse energy value: {0}")]
    ParseEnergyError(String),

    #[error("Failed to parse duration: {0}")]
    ParseDurationError(String),

    #[error("Invalid CSV format: {0}")]
    InvalidCsvFormat(String),

    #[error("Invalid JSON format: {0}")]
    InvalidJsonFormat(String),

    #[error("Invalid regex pattern: {0}")]
    InvalidPattern(String),

    #[error("Not enough snapshots to retrieve metrics")]
    NotEnoughSnapshots,
}

impl From<std::io::Error> for JouleProfilerError {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::NotFound => Self::RaplNotAvailable(err.to_string()),
            std::io::ErrorKind::PermissionDenied => Self::InsufficientPermissions,
            _ => Self::RaplReadError(err.to_string()),
        }
    }
}

impl JouleProfilerError {
    pub fn command_not_found(cmd: impl AsRef<str>) -> Self {
        Self::CommandNotFound(cmd.as_ref().to_string())
    }

    pub fn token_not_found(token: impl AsRef<str>) -> Self {
        Self::TokenNotFound(token.as_ref().to_string())
    }

    pub fn socket_not_found(socket: u32) -> Self {
        Self::SocketNotFound(socket)
    }
}
