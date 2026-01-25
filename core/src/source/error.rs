use std::fmt::Debug;

use thiserror::Error;

/// Errors that can occur when reading or aggregating metrics from a source.
///
/// This enum is used by all metric sources implementing [`MetricReader`](`crate::reader::MetricReader`)
/// to signal failures during measurement or iteration building.
///
/// # Variants
///
/// - `ErrorRetrievingCounters`: Failed to read counters from the source.
///
/// - `NoPhaseInIteration`: An iteration cannot be built because no phases
///   were collected from the source.
///
/// - `Rapl` (`RaplError`): An error occurred specifically in the Intel RAPL
///   source. This variant wraps the underlying `RaplError` using `#[from]`
///   for automatic conversion.
///
/// - `External` (boxed [`std::error::Error`]): An external or unknown error
///   from a custom source. The boxed trait object allows any error that is
///   `Send + Sync` to be propagated.
#[derive(Debug, Error)]
pub enum MetricSourceError {
    #[error("Error retrieving source counters")]
    ErrorRetrievingCounters,

    #[error("Cannot build iteration without at least one phase")]
    NoPhaseInIteration,

    #[error("External source error: {0}")]
    External(
        #[from]
        #[source]
        Box<dyn std::error::Error + Send + Sync>,
    ),
}
