# Adding a Custom Source

Joule Profiler is built on a modular architecture designed for extensibility. This allows users to integrate new metric sources such as custom hardware sensors, software counters, or external APIs without modifying the core profiling engine.

## Trait Implementation

To create a valid source for **Joule Profiler**, your struct must implement the `MetricReader` trait.  
Some provided methods are optional.

See the [source implementation example](../../examples/source-implementation.md) to understand with a minimal example how to implement a source.

```rs
pub trait MetricReader: Send + 'static {
    /// Type of metrics returned by the reader.
    type Type: MetricReaderTypeBound;

    /// Error type produced by the reader.
    type Error: MetricReaderErrorBound;

    //
    // Mandatory methods
    //

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

    //
    // Optional methods
    //

    /// Initialize the source before measurements
    fn init(&mut self) -> impl Future<Output = Result<(), Self::Error>> + Send;

    /// Cleanup or join logic after measurements
    fn join(&mut self) -> impl Future<Output = Result<(), Self::Error>> + Send;

    /// Reset internal counters
    fn reset(&mut self) -> impl Future<Output = Result<(), Self::Error>> + Send;
}
```

### Best Practices

When implementing a metric source, keep the `measure` method lightweight and fast; any heavy computation or data processing should be done in `to_metrics` to ensure measurements do not introduce overhead or slow down profiling.

## How to Register a Source

Once a source is implemented (see [Trait Implementation](trait-implementation.md)), adding it to the profiler is straightforward. You simply instantiate your source and register it with the `JouleProfiler` instance before starting to measure.

```rs
use joule_profiler::JouleProfiler;
use my_custom_source::MySource;

#[tokio::main]
async fn main() {
    // 1. Create the profiler
    let mut profiler = JouleProfiler::new();

    // 2. Instantiate your custom source
    let my_source = MySource::new();

    // 3. Register the source
    profiler.add_source(my_source);

    // 4. Start profiling
    let results = profiler.profile().await.unwrap();

    // 5. Use the results as you need
}
```