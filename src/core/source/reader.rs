use std::fmt::Debug;

use log::trace;

use crate::core::{aggregate::metric::Metrics, sensor::Sensors, source::error::MetricSourceError};

/// Bounds of the type used in a metric reader
pub trait MetricReaderType: Debug + Send + Default + Into<Metrics> {}

impl<T> MetricReaderType for T where T: Debug + Default + Send + Into<Metrics> {}

/// Trait implemented by metric readers
pub trait MetricReader: Send + 'static {
    /// Type of metrics returned by the reader
    type Type: MetricReaderType;

    /// Error type produced by the reader
    type Error: Debug + Into<MetricSourceError>;

    /// Measure the sensors metrics
    fn measure(&mut self) -> Result<(), Self::Error>;

    /// Retrieve current counters as the reader type
    fn retrieve_counters(&mut self) -> Result<Self::Type, Self::Error>;

    /// Return all sensors available from this reader
    fn get_sensors(&self) -> Result<Sensors, Self::Error>;

    /// Internal scheduler invoked by the accumulator periodically
    fn internal_scheduler(&mut self) -> impl Future<Output = Result<(), Self::Error>> + Send {
        trace!("Internal scheduler not implemented for this source");
        async { Ok(()) }
    }

    /// Enable or disable polling for this source
    fn set_polling(&mut self, _polling: bool) -> Result<(), Self::Error> {
        trace!("Polling not implemented for this source");
        Ok(())
    }
}