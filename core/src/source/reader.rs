use tokio::task::JoinHandle;

use crate::aggregate::Metrics;
use crate::sensor::Sensors;
use crate::source::{MetricReaderErrorBound, MetricReaderTypeBound, SourceEventEmitter};

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

    #[allow(clippy::type_complexity)]
    // Return type is intentionally explicit to avoid boxing or trait objects.
    fn run(
        &self,
        _eventer: SourceEventEmitter,
    ) -> impl Future<Output = Result<Option<JoinHandle<Result<(), Self::Error>>>, Self::Error>> + Send
    {
        async { Ok(None) }
    }

    /// Convert the metric reader data to metrics
    fn to_metrics(&self, result: Self::Type) -> Metrics;

    /// Get the name of the metric source
    fn get_name() -> &'static str;
}
