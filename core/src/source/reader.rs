use log::trace;

use crate::sensor::Sensors;
use crate::source::{MetricReaderErrorBound, MetricReaderTypeBound};

/// Trait implemented by a metric source reader.
///
/// This trait defines the interface that all metric sources must implement
/// to provide metrics to the Joule Profiler. It allows measuring, retrieving,
/// and enumerating sensors, and optionally supports a periodic scheduler.
///
/// # Associated Types
///
/// - `Type` ([`MetricReaderTypeBound`]): The type returned by the reader when
///   retrieving metrics. Typically represents a snapshot or collection of metrics.
/// - `Error` ([`MetricReaderErrorBound`]): The error type produced by the reader.
///
/// # Required Methods
///
/// - [`MetricReader::measure`] — Perform a measurement and update internal state.
/// - [`MetricReader::retrieve`] — Retrieve the current metrics collected by the reader.
/// - [`MetricReader::get_sensors`] — Return the list of sensors provided by this reader.
///
/// # Optional Methods
///
/// - [`MetricReader::scheduler`] — Internal periodic scheduler invoked by the accumulator.
///   By default, this is a no-op and returns a pending future. Implement this
///   if your source supports automatic periodic polling.
pub trait MetricReader: Send + 'static {
    /// Type of metrics returned by the reader.
    type Type: MetricReaderTypeBound;

    /// Error type produced by the reader.
    type Error: MetricReaderErrorBound;

    /// Measure the sensors metrics and update internal state
    fn measure(&mut self) -> Result<(), Self::Error>;

    /// Retrieve the current metrics as the reader type
    fn retrieve(&mut self) -> Result<Self::Type, Self::Error>;

    /// Return all sensors available from this reader
    fn get_sensors(&self) -> Result<Sensors, Self::Error>;

    /// Optional internal scheduler invoked by the accumulator periodically
    ///
    /// By default, this is a no-op returning a pending future.
    fn scheduler(&mut self) -> impl Future<Output = Result<(), Self::Error>> + Send {
        trace!("Internal scheduler not implemented for this source");
        async { Ok(()) }
    }

    fn has_scheduler(&self) -> bool {
        false
    }

    fn get_name() -> &'static str;
}
