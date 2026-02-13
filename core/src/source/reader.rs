use crate::aggregate::Metrics;
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
/// - [`MetricReader::to_metrics`] — Convert a snapshot to metrics. Implementing From<Self::Type> for Metrics isn't enough,
///   sometimes you need to use some information of the source to efficiently convert snapshots into metrics.
/// - [`MetricReader::reset`] — Reset the current counters of the source, used before measurements to remove the initialization overhead.
/// - [`MetricReader::get_name`] — Return the static name of the source.
///
/// # Optional Methods
///
/// - [`MetricReader::init`] — Source initialization logic if there is one, called before the measurements.
/// - [`MetricReader::join`] — Source destruction logic if there is one, called before the measurements (no Drop implementation because the source is reusable).
///   Calling init and join several times should always behave the same, the source shall be reset when joining.
/// - [`MetricReader::reset`] — Reset the source counters, called before the measurements, it is useful if a source implements polling and
///   measures have already been made before measurements.
pub trait MetricReader: Send + 'static {
    /// Type of metrics returned by the reader.
    type Type: MetricReaderTypeBound;

    /// Error type produced by the reader.
    type Error: MetricReaderErrorBound;

    /// Init the source if it implements custom logic underneath.
    fn init(&mut self, _pid: i32) -> impl Future<Output = Result<(), Self::Error>> + Send {
        async { Ok(()) }
    }

    /// Join the source if it implements custom logic underneath
    fn join(&mut self) -> impl Future<Output = Result<(), Self::Error>> + Send {
        async { Ok(()) }
    }

    /// Measure the sensors metrics and update internal state.
    fn measure(&mut self) -> impl Future<Output = Result<(), Self::Error>> + Send;

    /// Reset the source counters if it implements custom logic underneath.
    fn reset(&mut self) -> impl Future<Output = Result<(), Self::Error>> + Send {
        async { Ok(()) }
    }

    /// Retrieve the current metrics as the reader type.
    fn retrieve(&mut self) -> impl Future<Output = Result<Self::Type, Self::Error>> + Send;

    /// Return all sensors available from this reader.
    fn get_sensors(&self) -> Result<Sensors, Self::Error>;

    /// Convert the metric reader data to metrics.
    fn to_metrics(&self, result: Self::Type) -> Metrics;

    /// Get the name of the metric source.
    fn get_name() -> &'static str;
}
