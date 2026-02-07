# Trait Implementation

To create a valid source for **Joule Profiler**, your struct must implement the `MetricReader` trait.  
Some methods are **optional (default/no-op)**.

```rs
pub trait MetricReader: Send + 'static {
    /// Type of metrics returned by the reader.
    type Type: MetricReaderTypeBound;

    /// Error type produced by the reader.
    type Error: MetricReaderErrorBound;

    // --------------------------
    // Mandatory / Operational methods
    // --------------------------

    /// Perform a measurement and update internal state
    fn measure(&mut self) -> impl Future<Output = Result<(), Self::Error>> + Send;

    /// Retrieve the current metrics snapshot
    fn retrieve(&mut self) -> impl Future<Output = Result<Self::Type, Self::Error>> + Send;

    /// Return all sensors provided by this source
    fn get_sensors(&self) -> Result<Sensors, Self::Error>;

    /// Convert a snapshot into Joule Profiler metrics
    fn to_metrics(&self, result: Self::Type) -> Metrics;

    /// Return the static name of the metric source
    fn get_name() -> &'static str;

    // --------------------------
    // Optional / No-op methods
    // --------------------------

    /// Initialize the source before measurements
    fn init(&mut self) -> impl Future<Output = Result<(), Self::Error>> + Send;

    /// Cleanup or join logic after measurements
    fn join(&mut self) -> impl Future<Output = Result<(), Self::Error>> + Send;

    /// Reset internal counters
    fn reset(&mut self) -> impl Future<Output = Result<(), Self::Error>> + Send;
}
```

## Best Practices

When implementing a metric source, keep the `measure` method lightweight and fast; any heavy computation or data processing should be done in `to_metrics` to ensure measurements do not introduce overhead or slow down profiling.

# Minimal Example

Implementing a new metric source in **Joule Profiler** is straightforward. By implementing the `MetricReader` trait, you only need to define the core measurement logic (`measure`, `retrieve`, `get_sensors`, `to_metrics`, `get_name`) and optionally override `init`, `join`, or `reset` if your source requires setup, polling logic, cleanup, or counter resets. This design makes it easy to add new sources without boilerplate.

```rs
use joule_profiler_core::{
    sensor::{Sensor, Sensors},
    source::{MetricReader, MetricSourceError},
    types::{Metric, Metrics},
};

#[derive(Default)]
pub struct MySource {
    count: u64,
}

impl MySource {
    pub fn new() -> Self {
        Self::default()
    }
}

impl MetricReader for MySource {
    type Type = u64;

    type Error = MetricSourceError;

    async fn measure(&mut self) -> Result<(), Self::Error> {
        self.count += 1;
        Ok(())
    }

    async fn retrieve(&mut self) -> Result<Self::Type, Self::Error> {
        let count = self.count;
        self.count = 0;
        Ok(count)
    }

    async fn reset(&mut self) -> Result<(), Self::Error> {
        self.count = 0;
        Ok(())
    }

    fn get_sensors(&self) -> Result<Sensors, Self::Error> {
        let sensors = vec![Sensor {
            name: "count".into(),
            source: Self::get_name().into(),
            unit: "count".into(),
        }];
        Ok(sensors)
    }

    fn to_metrics(&self, count: u64) -> Metrics {
        vec![Metric {
            name: "count".into(),
            source: Self::get_name().into(),
            unit: "count".into(),
            value: count,
        }]
    }

    fn get_name() -> &'static str {
        "my_source"
    }
}
```