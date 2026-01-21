# Architecture

**Joule Profiler** is designed to minimize measurement overhead while maintaining high performance, modularity and extensibility, and a strong separation of concerns.
To achieve this, the project adopts a flexible domain-driven architecture centered around a core domain.

This architecture enables:
- Efficient asynchronous scheduling
- Low-overhead metrics collection
- Easy integration of new metric sources
- User-defined metrics sources extensions without modifying the core codebase

## High-Level Design

At a high level, Joule Profiler is composed of three main layers:
- CLI / Configuration - Responsible for user input, configuration parsing, and startup wiring.
- Core Module - Contains all domain logic: orchestration, aggregation, scheduling, and result modeling.
- Sources Module - Implementations of the API traits for user-defined metrics sources or displayers. 

## Domain-Driven Architecture

The `core` module represents the domain boundary of the profiler.
It does not depend on any concrete metric source or output format.

Key properties:
- No direct filesystem, OS, or hardware coupling
- All interactions go through traits
- Clear separation between what is measure and how it is measure

This design allows the profiler to evolve without breaking existing sources or outputs.

## Abstraction

Metrics sources implement the `MetricReader` trait:
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

- Associated types (Type, Error)
    - Allow each source to define its own data representation
    - Allow users to easily implement metrics sources
    - Avoid boxing or type erasure during measurement
- Optional scheduler
    - Sources can implement asynchronous polling using tokio
    - Zero-cost for sources that do not require polling (pending future)

### Dynamic to Static Resolution

During startup, metrics sources are handled dynamically using the MetricSource trait.

This allows:
- Heterogeneous sources in a single collection
- Extensibility (plugins, user-defined sources)
- Configuration-driven source selection (in the future)



Sources may implement asynchronous polling

Zero-cost for sources that do not require polling