# Using the library

Joule Profiler can be used directly as a Rust **library**, allowing full programmatic control over measurement, configuration, and result handling.  
This approach is ideal for users who want **custom profiling**, integrate profiling in tests or benchmarks, or manipulate the results directly in Rust.

## Adding Joule Profiler to your project

Add the following to your `Cargo.toml`:

```toml
[dependencies]
joule-profiler-core = { path = "../joule-profiler-core" }
source-rapl = { path = "../source-rapl" }
```

> Adjust the paths according to your project layout.

> The profiler core crate will be published on **crates.io** soon, to be usable easily with cargo.

## Basic Usage

```rust
use joule_profiler_core::{JouleProfiler, config::ProfileConfigBuilder};
use source_rapl::Rapl;

#[tokio::main]
async fn main() {
    // Create the profiler
    let mut profiler = JouleProfiler::new();

    // Add sources (RAPL, NVML, Perf, or custom sources)
    let rapl = Rapl::with_default_path().unwrap();
    profiler.add_source(rapl);

    // Configure the profiling session
    let config = ProfileConfigBuilder::default()
        .cmd(vec!["sleep".into(), "1".into()])
        .iterations(1)
        .build()
        .unwrap();

    // Run the profiling session
    let results = profiler.run_phases(&config).await.unwrap();

    // Print or process the results
    println!("{:?}", results);
}
```

## Adding Custom Sources

Custom sources implementing `MetricReader` can be added to the profiler:

```rust
use joule_profiler_core::JouleProfiler;
use my_source::MySource;

let mut profiler = JouleProfiler::new();
profiler.add_source(MySource::new());
```

> The profiler will automatically schedule all sources concurrently and handle measurements.

## Configuring Phases and Iterations

The library allows fine-grained control over **iterations** and **phases**:

```rust
use joule_profiler_core::config::ProfileConfigBuilder;

let config = ProfileConfigBuilder::default()
    .cmd(vec!["./my_program".into()])
    .iterations(5) // repeat measurement 5 times
    .build()
    .unwrap();
```

- **Iterations**: repeat the measurement multiple times to improve accuracy.
- **Phases**: if your program outputs phase tokens, the profiler can detect them and measure per-phase metrics, a default phase called `START -> END` is initialized by default.

## Retrieving Sensors

You can query all sensors available from your sources:

```rust
let sensors = profiler.run_list_sensors().unwrap();
for sensor in sensors {
    println!("{} ({}) [{}]", sensor.name, sensor.source, sensor.unit);
}
```

- Useful to know which sensors (CPU cores, DRAM, GPU) are accessible before profiling.

## Processing Results

`run_phases` returns structured data that can be iterated or converted to other formats:

```rust
for phase_result in results.iter() {
    for metric in &phase_result.metrics {
        println!("{}: {} {}", metric.source, metric.value, metric.unit);
    }
}
```

- **Metrics** include: `name`, `source`, `unit`, and `value`.
- Calculations and conversions are performed **after the measurement**, keeping runtime overhead minimal.

## Notes

- The library is **fully asynchronous**, ensure you use a Tokio runtime.

---

## Summary

Using Joule Profiler as a library allows:

- Programmatic profiling of Rust applications.
- Fine-grained control over sources, iterations, and phases.
- Seamless integration of custom metric sources.
- Minimal measurement overhead with deterministic scheduling.
- Structured results for analysis, logging, or further processing.