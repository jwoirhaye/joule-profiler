use crate::sensor::Sensor;
use crate::source::MetricReader;
use crate::unit::{MetricUnit, Unit, UnitPrefix};
use crate::{aggregate::Metrics, sensor::Sensors};
use std::fmt::Display;
use std::sync::{Arc, Mutex};

#[derive(Debug, Default, Clone)]
pub struct Counts {
    pub init: usize,
    pub join: usize,
    pub measure: usize,
    pub reset: usize,
    pub retrieve: usize,
}

pub struct MockMetricReader {
    pub counts: Arc<Mutex<Counts>>,
    pub measure_error: Option<String>,
    pub retrieve_error: Option<String>,
}

impl MockMetricReader {
    pub fn new() -> Self {
        Self {
            counts: Arc::new(Mutex::new(Counts::default())),
            measure_error: None,
            retrieve_error: None,
        }
    }

    pub fn unit() -> MetricUnit {
        MetricUnit {
            prefix: UnitPrefix::None,
            unit: Unit::Count,
        }
    }

    pub fn sensors() -> Sensors {
        vec![Sensor {
            name: "MockSensor".to_string(),
            source: "Mock".to_string(),
            unit: Self::unit(),
        }]
    }
}

impl MetricReader for MockMetricReader {
    type Type = ();
    type Error = MockMetricReaderError;

    async fn init(&mut self, _pid: i32) -> Result<(), Self::Error> {
        self.counts.lock().unwrap().init += 1;
        Ok(())
    }

    async fn join(&mut self) -> Result<(), Self::Error> {
        self.counts.lock().unwrap().join += 1;
        Ok(())
    }

    async fn measure(&mut self) -> Result<(), Self::Error> {
        self.counts.lock().unwrap().measure += 1;
        match &self.measure_error {
            Some(e) => Err(MockMetricReaderError(e.clone())),
            None => Ok(()),
        }
    }

    async fn reset(&mut self) -> Result<(), Self::Error> {
        self.counts.lock().unwrap().reset += 1;
        Ok(())
    }

    async fn retrieve(&mut self) -> Result<Self::Type, Self::Error> {
        self.counts.lock().unwrap().retrieve += 1;
        match &self.retrieve_error {
            Some(e) => Err(MockMetricReaderError(e.clone())),
            None => Ok(()),
        }
    }

    fn get_sensors(&self) -> Result<Sensors, Self::Error> {
        Ok(Self::sensors())
    }

    fn to_metrics(&self, _: Self::Type) -> Result<Metrics, Self::Error> {
        Ok(Metrics::default())
    }

    fn get_name() -> &'static str {
        "mock"
    }
}

#[derive(Debug)]
pub struct MockMetricReaderError(String);

impl Display for MockMetricReaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for MockMetricReaderError {}
