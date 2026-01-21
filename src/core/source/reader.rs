use futures::future::pending;
use log::trace;

use crate::core::{
    sensor::Sensors,
    source::types::{MetricReaderErrorBound, MetricReaderTypeBound},
};

/// Trait implemented by metric readers
pub trait MetricReader: Send + 'static {
    /// Type of metrics returned by the reader
    type Type: MetricReaderTypeBound;

    /// Error type produced by the reader
    type Error: MetricReaderErrorBound;

    /// Measure the sensors metrics
    fn measure(&mut self) -> Result<(), Self::Error>;

    /// Retrieve current counters as the reader type
    fn retrieve(&mut self) -> Result<Self::Type, Self::Error>;

    /// Return all sensors available from this reader
    fn get_sensors(&self) -> Result<Sensors, Self::Error>;

    /// Internal scheduler invoked by the accumulator periodically
    fn scheduler(&mut self) -> impl Future<Output = Result<(), Self::Error>> + Send {
        trace!("Internal scheduler not implemented for this source");
        pending()
    }
}
