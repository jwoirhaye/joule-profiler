use thiserror::Error;

#[derive(Debug, Error)]
pub enum RaplError {
    #[error("Intel RAPL not available: {0}")]
    RaplNotAvailable(String),

    #[error("No RAPL domains found")]
    NoDomains,

    #[error("Failed to read RAPL domain: {0}")]
    RaplReadError(String),

    #[error("Invalid RAPL domain path: {0}")]
    InvalidRaplPath(String),

    #[error("Invalid socket specification: {0}")]
    InvalidSocketSpec(String),

    #[error("Socket {0} not found in available RAPL domains")]
    SocketNotFound(u32),

    #[error("Command not found: {0}")]
    CommandNotFound(String),

    #[error("Unsupported operating system: {0}. Only Linux is supported")]
    UnsupportedOS(String),

    #[error("Insufficient permissions to access RAPL. Try running with sudo")]
    InsufficientPermissions,

    #[error("Unknown domain {0}")]
    UnknownDomain(String),

    #[error("Failed to open domain counter {0}")]
    FailToOpenDomainCounter(String),

    #[error("Invalid event format for domain {0}")]
    InvalidEventFormat(String),

    #[error("Perf paranoid error")]
    PerfParanoid(
        #[from]
        #[source]
        PerfParanoidError,
    ),

    #[error(transparent)]
    IoError(std::io::Error),

    #[error("Failed to parse energy value")]
    ParseEnergyError(
        #[from]
        #[source]
        std::num::ParseIntError,
    ),

    #[error("Failed to parse domain scale")]
    ParseDomainScale(
        #[from]
        #[source]
        std::num::ParseFloatError,
    ),
}

impl From<std::io::Error> for RaplError {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::NotFound => RaplError::RaplNotAvailable(err.to_string()),
            std::io::ErrorKind::PermissionDenied => RaplError::InsufficientPermissions,
            _ => RaplError::IoError(err),
        }
    }
}

#[derive(Debug, Error)]
pub enum PerfParanoidError {
    #[error(
        "perf_event_paranoid level too high: {0}, try with root privileges or set paranoid level to zero"
    )]
    LevelTooHigh(u8),

    #[error("perf_event_paranoid file not found")]
    NotFound,

    #[error("perf_event_paranoid not readable: {0}")]
    PermissionDenied(String),

    #[error(transparent)]
    IoError(std::io::Error),

    #[error("Failed to parse paranoid level")]
    ParseParanoidLevelError(
        #[from]
        #[source]
        std::num::ParseIntError,
    ),
}
