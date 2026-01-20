use std::fmt::Debug;

use thiserror::Error;

use crate::{
    core::{orchestrator::error::OrchestratorError, profiler::error::JouleProfilerError},
    sources::rapl::error::RaplError,
};

#[derive(Debug, Error)]
pub enum MetricSourceError {
    #[error("RAPL source failed")]
    Rapl {
        #[source]
        err: RaplError,
    },
}

impl From<MetricSourceError> for OrchestratorError {
    fn from(err: MetricSourceError) -> Self {
        Self::SourceError { err }
    }
}

impl From<MetricSourceError> for JouleProfilerError {
    fn from(err: MetricSourceError) -> Self {
        Self::Source { err }
    }
}
