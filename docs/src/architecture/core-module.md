# Core Module

The Core Module is the engine of **Joule Profiler**.

It defines the profiling model, coordinates measurements, and produces structured results.

## Responsibilities

The **Core Module** is responsible for:
- Managing the measurements
- Coordinating metric sources
- Collecting and aggregating results
- Exposing a stable and easy to use API

It contains no user interface logic and no hardware-specific code.

## Domain Concepts

The core models profiling in terms of a few simple concepts:
- Iterations – Repeated executions of a measured program
- Phases – Logical segments of work inside an iteration
- Measurements – Raw data collected during execution
- Metrics – Aggregated data of a phase
- Results – Final, aggregated outputs

These concepts are intentionally generic so they can support many kinds of measurements without change.

## Orchestration

The core coordinates all metric sources through a central orchestrator.

The orchestrator:
- Starts and stops measurements
- Signals phase and iteration boundaries
- Collects finalized data from sources
- Produces a unified result set

Metric sources never interact with each other directly.
All coordination flows through the core and is hidden from the sources.

## Modularity and Extensibility

This design ensure that the implementation of new metric sources, output formats, or CLI features should not require changes to the core domain logic.
This ensures that existing workflows remain stable while the ecosystem grows.

## Measurement Lifecycle

During a profiling session, the core controls when measurements begin and end.

Measurements are associated with phase boundaries and accumulated across iterations.
All raw data is collected first, then processed once the measurements complete.

This separation ensures consistent results and reduce the measurements overhead.
