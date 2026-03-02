use thiserror::Error;

use crate::event::Event;

#[derive(Debug, Error)]
pub enum PerfEventError {
    #[error("{0}")]
    Err(String),

    #[error("{0}")]
    IoError(
        #[from]
        #[source]
        std::io::Error,
    ),

    #[error("Error reading counter {0}")]
    ErrorReadingCounter(Event),

    #[error("Not enough measures to compute perf counters differences")]
    NotEnoughSamples,
}
