use thiserror::Error;

use crate::core::source::error::MetricSourceError;

#[derive(Debug, Error)]
pub enum RaplError {
    #[error("Intel RAPL not available at {0}")]
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

    #[error(transparent)]
    IoError(std::io::Error),

    #[error("failed to parse energy value")]
    ParseEnergyError(#[source] std::num::ParseIntError),
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

impl From<RaplError> for MetricSourceError {
    fn from(err: RaplError) -> Self {
        Self::Rapl { err }
    }
}
