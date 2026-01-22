# Architecture

**Joule Profiler** is designed to minimize measurement overhead while maintaining high performance, modularity, extensibility, and a strong separation of concerns.  
To achieve this, the project adopts a flexible domain-driven architecture centered around a core domain.

This architecture enables:
- Efficient asynchronous scheduling
- Low-overhead metric collection
- Easy integration of new metric sources
- User-defined metric source extensions without modifying the core codebase

## High-Level Design

At a high level, Joule Profiler is composed of three main layers:
- **CLI / Configuration** – Responsible for user input, configuration parsing, and startup wiring
- **Core Module** – Contains all domain logic: orchestration, aggregation, scheduling, and result modeling
- **Sources Module** – Implementations of the API traits for user-defined metric sources or displayers

---

## Domain-Driven Architecture

The `core` module represents the domain boundary of the profiler.  
It does not depend on any concrete metric source or output format.

Key properties:
- Clear separation between what is measured and how it is measured, expressed through a common interface
- No direct filesystem, OS, or hardware coupling
- All interactions with external systems are performed through traits

This design allows the profiler to evolve independently of existing sources or outputs.

---

## Abstraction

Metric sources implement the `MetricReader` trait:
```rs
pub trait MetricReader {
    type Type;
    type Error;

    fn measure(&mut self) -> Result<(), Self::Error>;
    fn retrieve(&mut self) -> Result<Self::Type, Self::Error>;
    fn get_sensors(&self) -> Result<Sensors, Self::Error>;
    async fn scheduler(&mut self) -> Result<(), Self::Error>; // Optional
}
```

This trait defines the contract expected by the domain from any metric reader implementation.

### Key Characteristics

#### Associated types (`Type`, `Error`)

- Allow each source to define its own data representation
- Simplify the implementation of custom metric sources
- Avoid boxing or type erasure during measurement

#### Optional scheduler

- Sources may implement asynchronous polling using Tokio
- Sources that do not require polling incur no overhead, as the default implementation is a no-op

### Dynamic to Static Resolution

During startup, metric sources are handled dynamically through the `MetricSource` trait, which is implemented by the `MetricAccumulator<R: MetricReader>` structure for each concrete `MetricReader` type.

All types implementing `MetricReader` can be converted into `Box<dyn MetricSource>`, enabling dynamic initialization and configuration.

This wrapping allows sources to be initialized dynamically and resolved into their concrete, monomorphized types before metric collection starts.  
As a result, dynamic dispatch is confined to initialization, and metric collection itself incurs zero overhead from dynamic typing.

This design also enables:

- Heterogeneous metric sources within a single collection
- Extensibility through plugins and user-defined sources
- Independent configuration of each metric source

---

## Source Orchestration

Metric sources are orchestrated by the `SourceOrchestrator`, which is responsible for:
- Spawning one asynchronous worker per metric source
- Broadcasting lifecycle and measurement events to all sources
- Coordinating polling, measurement phases, and iterations
- Collecting and merging results once profiling completes

Each metric source runs in its own Tokio task and communicates exclusively through asynchronous channels, ensuring isolation between sources and minimal synchronization overhead.

### Worker Lifecycle

For each source:
1. A bounded channel is created to send control events
2. The source is moved into a dedicated worker task
3. The worker processes events until it is explicitly joined

Workers are joined gracefully, allowing each source to return both its collected results and a reusable instance of itself.

---

## Event-Driven Control Flow

Sources are driven by a small set of domain events (`SourceEvent`) broadcast by the orchestrator:

- `Measure` — perform a metric measurement
- `NewPhase` — finalize the current phase and snapshot counters
- `NewIteration` — finalize the current iteration
- `StartScheduler` — enable asynchronous polling
- `StopScheduler` — pause asynchronous polling
- `JoinWorker` — terminate the worker and return results

This event-based design decouples orchestration logic from metric collection and allows all sources to be controlled uniformly, regardless of their internal implementation.

---

## Iterations and Phases Tracking

Each source worker is backed by a `MetricAccumulator<R>`, where `R` is a concrete `MetricReader`.

The accumulator is responsible for:
- Tracking elapsed time between measurements
- Grouping measurements into phases
- Grouping phases into iterations
- Converting raw measurements into domain-level results

Iterations and phases are accumulated incrementally and finalized only when explicitly requested by the orchestrator.

---

## Asynchronous Scheduling

Metric readers may optionally implement an internal asynchronous scheduler via `MetricReader::scheduler`.

When enabled:
- The scheduler future is polled only while the source is in a running state
- Scheduler execution is interleaved with event handling using `tokio::select!`
- Errors from the scheduler are immediately propagated and terminate the worker

Even readers without a scheduler still participate in the loop via a default pending future, which introduces minimal but non-zero overhead.

This design allows time-based or event-based sampling without introducing background threads or global timers.

---

## Result Collection

When profiling completes, the orchestrator:
1. Sends a `JoinWorker` event to all sources
2. Awaits all worker tasks
3. Collects each source’s `SensorResult`
4. Merges results into a single aggregated view

Each worker returns both its results and a freshly reset metric source, allowing sources to be reused across profiling runs without reinitialization.

---

## Error Handling and Failure Propagation

The profiler adopts a fail-fast error model.

Any error occurring in a metric source immediately stops the entire profiling process.  
This includes:
- Errors returned by a `MetricReader`
- Failures in a source scheduler
- Worker task failures or unexpected disconnections

When an error is detected:
1. The failing source reports the error to the orchestrator
2. The orchestrator stops event propagation
3. The profiling session terminates with the reported error

Errors originating from metric readers are wrapped into `MetricSourceError` and propagated unchanged through the core domain, ensuring that failure causes remain explicit and traceable.

This design favors correctness and result integrity over partial measurements, preventing the profiler from producing incomplete or inconsistent data.

---

## Design Summary

This architecture provides:
- One-task-per-source isolation
- Event-driven, deterministic control over measurement
- Zero dynamic dispatch in the measurement hot path
- Optional asynchronous polling without background overhead
- Strong fault isolation and graceful shutdown

The result is a profiler that remains extensible and flexible while preserving predictable performance characteristics.
