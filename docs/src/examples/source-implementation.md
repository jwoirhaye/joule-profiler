# Source Implementation

Implementing a new metric source in **Joule Profiler** is straightforward. By implementing the `MetricReader` trait, you only need to define the core measurement logic (`measure`, `retrieve`, `get_sensors`, `to_metrics`, `get_name`) and optionally override `init`, `join`, or `reset` if your source requires setup, polling logic, cleanup, or counter resets. This design makes it easy to add new sources without boilerplate.

```rs
use joule_profiler_core::{
    sensor::{Sensor, Sensors},
    source::{MetricReader, MetricSourceError},
    types::{Metric, Metrics},
    unit::{MetricUnit, Unit, UnitPrefix},
};

const MY_SOURCE_UNIT: MetricUnit = MetricUnit {
    prefix: UnitPrefix::None,
    unit: Unit::Count,
};

#[derive(Default)]
struct MySource {
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

    fn get_sensors(&self) -> Result<Sensors, Self::Error> {
        let sensors = vec![Sensor {
            name: "value".into(),
            source: "my_source".into(),
            unit: MY_SOURCE_UNIT,
        }];
        Ok(sensors)
    }

    fn to_metrics(&self, count: u64) -> Metrics {
        let metric = Metric {
            name: "value".into(),
            source: "my_source".into(),
            unit: MetricUnit {
                prefix: UnitPrefix::None,
                unit: Unit::Count,
            },
            value: count,
        };
        vec![metric]
    }

    fn get_name() -> &'static str {
        "my_source"
    }
}
```