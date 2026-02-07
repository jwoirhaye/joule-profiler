# Adding a Custom Source

Joule Profiler is built on a modular architecture designed for extensibility. This allows users to integrate new metric sources such as custom hardware sensors, software counters, or external APIs without modifying the core profiling engine.

## How to Register a Source

Once a source is implemented (see [Trait Implementation](trait-implementation.md)), adding it to the profiler is straightforward. You simply instantiate your source and register it with the `JouleProfiler` instance before starting the profile.

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