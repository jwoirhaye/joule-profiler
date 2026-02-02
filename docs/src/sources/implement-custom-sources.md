# Implementing a custom Metric Source

Joule Profiler is designed to be **extensible**, allowing users to add new metric sources without modifying the core profiler.  
Each source implements a **defined API** that the core uses to schedule, measure, and retrieve metrics.
All metric sources must implement the **async `MetricReader` trait** to allow concurrent measurement using Tokio.

## Key Requirements

A metric source must:

1. **Provide a name** – a static identifier for the source.  
2. **Expose available sensors** – so the profiler knows what can be measured.  
3. **Measure metrics** – collect the current values of the sensors.  
4. **Return results in a structured type** – for aggregation and post-processing.  
5. **Optionally implement asynchronous sampling** – for precise, time-based measurement.

> All calculations or transformations should be deferred until after measurements, to maintain minimal overhead during profiling, you should implement the rawest data structure possible to store your metrics.

## Implementing a Source

1. **Create a new Rust module** for your source (e.g., `my_source`).  
2. **Define a metric type** that will hold your measurements.  
3. **Implement the `MetricReader` trait** for your source. This includes:

- `init` – optional asynchronous initialization of the source (can be used to implement polling strategy, see Rapl source implementation for example).
- `measure` – collect the metrics.
- `retrieve` – return the collected metrics.
- `get_sensors` – return the list of sensors available from this source.
- `to_metrics` – convert the collected data into the profiler's `Metrics` type.
- `get_name` – return the static name of the source.

### Example

```rust
use joule_profiler_core::MetricReader;
use joule_profiler_core::Metrics;

pub struct MySource {
    // Internal state or handles to hardware counters
}

pub struct MyMetrics {
    // Data fields for your measurements
}

impl MetricReader for MySource {
    type Type = MyMetrics;
    type Error = MyError;

    async fn init(&mut self) -> impl Future<Output = Result<(), Self::Error>> + Send {
        // Optional initialization logic
        Ok(())
    }

    async fn join(&mut self) -> impl Future<Output = Result<(), Self::Error>> + Send {
        // Optional joining logic (you could also use the Drop trait)
        Ok(())
    }

    async fn measure(&mut self) -> impl Future<Output = Result<(), Self::Error>> + Send {
        // Read hardware counters or compute metrics
        Ok(())
    }

    async fn retrieve(&mut self) -> impl Future<Output = Result<Self::Type, Self::Error>> + Send {
        // Return collected metrics for this phase
        Ok(MyMetrics { /* ... */ })
    }

    fn get_sensors(&self) -> Result<Sensors, Self::Error> {
        // Return list of available sensors
    }

    fn to_metrics(&self, result: Self::Type) -> Metrics {
        // Convert your metrics type to the profiler's Metrics
    }

    fn get_name() -> &'static str {
        "my-source"
    }
}
```

## Best Practices

- **Keep measurement fast** – avoid heavy computation in `measure`, offload post-processing to `to_metrics`.
- **Use asynchronous polling** if your source requires frequent updates.
- **Return structured errors** – any hardware or IO errors should be propagated clearly to maintain fail-fast behavior.
- **Support multiple sensors** – if the hardware exposes multiple domains (CPU cores, GPU devices), represent them in your metric type.
- **Use async for any I/O** to avoid blocking other sources.

## Adding Your Source to the Profiler

Once your source is implemented:

```rust
let mut profiler = JouleProfiler::new();
let my_source = MySource::new();
profiler.add_source(my_source);
```

The profiler will then handle measurement, orchestration, and result aggregation automatically.

## Summary

By implementing a new metric source efficiently:

- You extend the profiler features without touching to the core module.
- Measurements remain low-overhead and concurrent.
- Post-processing is handled consistently.

This approach ensures **accurate, reproducible profiling** while keeping the system modular and maintainable.