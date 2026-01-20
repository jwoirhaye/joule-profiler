use std::fmt::Debug;

use log::trace;

use crate::core::{aggregate::metric::Metrics, sensor::Sensors, source::error::MetricSourceError};

pub trait MetricReaderType: Debug + Send + Default + Into<Metrics> {}

impl<T> MetricReaderType for T where T: Debug + Default + Send + Into<Metrics> {}

pub trait MetricReader: Send + 'static {
    type Type: MetricReaderType;
    type Error: Debug + Into<MetricSourceError>;

    /// Measure the sensors metrics.
    fn measure(&mut self) -> Result<(), Self::Error>;

    fn retrieve_counters(&mut self) -> Result<Self::Type, Self::Error>;

    fn get_sensors(&self) -> Result<Sensors, Self::Error>;

    fn internal_scheduler(&mut self) -> impl Future<Output = Result<(), Self::Error>> + Send {
        trace!("Internal scheduler not implemented for this source");
        async { Ok(()) }
    }

    fn set_polling(&mut self, _polling: bool) -> Result<(), Self::Error> {
        trace!("Polling not implemented for this source");
        Ok(())
    }
}
