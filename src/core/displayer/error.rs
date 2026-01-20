use thiserror::Error;

use crate::core::profiler::error::JouleProfilerError;

#[derive(Debug, Error)]
pub enum DisplayerError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error("Not implemented for this format")]
    NotImplementedForFormat,

    #[error("Serialization error")]
    SerializeError(#[from] serde_json::Error),
}

impl From<DisplayerError> for JouleProfilerError {
    fn from(err: DisplayerError) -> Self {
        Self::Displayer { err }
    }
}
