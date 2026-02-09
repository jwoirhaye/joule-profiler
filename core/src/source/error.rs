use std::fmt::Debug;

use thiserror::Error;

/// Errors that can occur when reading or aggregating metrics from a source.
///
/// This enum is used by all metric sources implementing [`MetricReader`](`super::MetricReader`)
/// to signal failures during measurement or iteration building.
///
/// # Variants
///
/// - `ErrorRetrievingCounters`: Failed to read counters from the source.
///
/// - `NoPhaseInIteration`: An iteration cannot be built because no phases
///   were collected from the source.
///
/// - `SourceError` (boxed [`std::error::Error`]): An external or unknown error
///   from a custom source. The boxed trait object allows any error that is
///   `Send + Sync` to be propagated.
#[derive(Debug, Error)]
pub enum MetricSourceError {
    #[error("Error retrieving source counters")]
    ErrorRetrievingCounters,

    #[error("Cannot build iteration without at least one phase")]
    NoPhaseInIterationError,

    #[error(transparent)]
    SourceError(#[from] Box<dyn std::error::Error + Send + Sync>),
}

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
