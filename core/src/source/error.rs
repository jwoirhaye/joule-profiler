use std::fmt::Debug;
use thiserror::Error;

/// Errors that can occur when reading or aggregating metrics from a source.
///
/// This enum is used by all metric sources implementing [`MetricReader`](`super::MetricReader`)
/// to signal failures during measurement or iteration building.
#[derive(Debug, Error)]
pub enum MetricSourceError {
    /// The source failed to retrieve its internal counters.
    #[error("Error retrieving source counters")]
    ErrorRetrievingCounters,

    /// An iteration cannot be built because no phases were recorded.
    #[error("Cannot build iteration without at least one phase")]
    NoPhaseInIterationError,

    /// Error propagated from a custom metric source.
    #[error(transparent)]
    SourceError(#[from] Box<dyn std::error::Error + Send + Sync>),
}

/// Converts any compatible error into a [`MetricSourceError`].
///
/// Implemented for all types that are [`std::error::Error`] + [`Send`] + [`Sync`] + `'static`,
/// wrapping them in [`MetricSourceError::SourceError`].
pub trait IntoMetricSourceError {
    fn into_metric_source_error(self) -> MetricSourceError;
}

impl<T> IntoMetricSourceError for T
where
    T: std::error::Error + Send + Sync + 'static,
{
    fn into_metric_source_error(self) -> MetricSourceError {
        MetricSourceError::SourceError(Box::new(self))
    }
}
