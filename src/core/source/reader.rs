use std::fmt::Debug;

use log::trace;

use crate::core::{metric::Metrics, sensor::Sensors, source::error::MetricSourceError};

pub trait MetricReaderTypeBound: Debug + Send + Default + Into<Metrics> {}

impl<T> MetricReaderTypeBound for T where T: Debug + Send + Default + Into<Metrics> {}

pub trait MetricReader: Send + 'static {
    type Type: MetricReaderTypeBound;
    type Error: Debug + Into<MetricSourceError>;

    /// Measure the sensors metrics.
    fn measure(&mut self) -> Result<(), Self::Error>;

    fn retrieve_counters(&mut self) -> Result<(Self::Type, u64), Self::Error>;

    fn get_sensors(&self) -> Result<Sensors, Self::Error>;

    fn internal_scheduler(&mut self) -> impl Future<Output = Result<(), Self::Error>> + Send {
        trace!("Internal scheduler not implemented for this source");
        async { Ok(()) }
    }

    fn get_measure_count(&self) -> u64 {
        trace!("get_measure_count not implemented for this source");
        0
    }

    fn set_polling(&mut self, _polling: bool) -> Result<(), Self::Error> {
        trace!("Polling not implemented for this source");
        Ok(())
    }
}
