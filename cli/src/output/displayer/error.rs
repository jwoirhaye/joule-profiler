use thiserror::Error;

/// Error type for display operations in JouleProfiler.
///
/// This enum represents errors that can occur when displaying or exporting
/// profiler results, whether to the terminal, JSON, or CSV.
///
/// # Variants
///
/// - `NotImplementedForFormat`: The requested display format does not support
///   the attempted operation (e.g., trying to list sensors in CSV output).
///
/// - `IoError` ([`std::io::Error`]): I/O error occurred during display,
///   e.g., writing to a file failed.
///
/// - `SerializeError` ([`serde_json::Error`]): Serialization error occurred
///   when exporting to JSON format.
#[derive(Debug, Error)]
pub enum DisplayerError {
    #[error("Not implemented for this format")]
    NotImplementedForFormat,

    #[error("I/O error")]
    IoError(
        #[from]
        #[source]
        std::io::Error,
    ),

    #[error("Displayer error: {0}")]
    OutputFormatError(
        #[from]
        #[source]
        Box<dyn std::error::Error + Send + Sync>,
    ),

    #[error("No iteration results to print")]
    NoIterationFound,
}

pub trait IntoDisplayerError {
    fn into_displayer_error(self) -> DisplayerError;
}

impl<T> IntoDisplayerError for T
where
    T: std::error::Error + Send + Sync + 'static,
{
    fn into_displayer_error(self) -> DisplayerError {
        DisplayerError::OutputFormatError(Box::new(self))
    }
}
