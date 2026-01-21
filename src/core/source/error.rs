use std::fmt::Debug;

use thiserror::Error;

use crate::sources::rapl::error::RaplError;

#[derive(Debug, Error)]
pub enum MetricSourceError {
    #[error("Error retrieving source counters")]
    ErrorRetrievingCounters,

    #[error("Cannot build iteration without at least one phase")]
    NoPhaseInIteration,

    #[error("Rapl error")]
    Rapl(
        #[from]
        #[source]
        RaplError,
    ),

    #[error("External source error: {0}")]
    External(
        #[from]
        #[source]
        Box<dyn std::error::Error + Send + Sync>,
    ),
}
