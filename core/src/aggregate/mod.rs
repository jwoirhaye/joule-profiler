//! Core aggregation module for JouleProfiler.
//!
//! This module provides structures and utilities to aggregate metrics
//! collected from various sources. It organizes data into iterations, phases,
//! and sensor-level results, and converts raw measurements into a unified format.
//!
//! # Public API
//!
//! Only the following types are publicly accessible under this module:
//! - [`Metric`] - Represents an individual metric measurement.
//! - [`Metrics`] - Represents a collection of metrics.
//!
//! **Note:** Metrics are only instantiated *after* the measurements have finished,
//! in order to avoid adding any runtime overhead during data collection.
//!
//! # Submodules (internal)
//!
//! - `iteration` - Defines `SensorIteration` for handling metric measurements over time.
//! - `phase` - Defines `SensorPhase` and phase-based profiling logic.
//! - `sensor_result` - Defines `SensorResult` which aggregates multiple sensor iterations.
//! - `metric` - Internal module containing the definitions of [`Metric`] and [`Metrics`].
//!
//! # Usage
//!
//! Metrics are collected from different sources and then aggregated into
//! a unified format. Typically, the flow is:
//! 1. Collect raw metrics from a source (without instantiating [`Metric`] objects).
//! 2. Wrap the collected data into a `SensorPhase` to include additional information like duration.
//! 3. Wrap phases into `SensorIteration` to include iteration-level data like polling frequency.
//! 4. Aggregate all iterations into `SensorResult`.
//! 5. Access raw metrics via [`Metric`] and [`Metrics`] after the measurements are finished.
//!
//! # Examples
//!
//! ```no_run
//! use joule_profiler_core::aggregate::{Metric, Metrics};
//!
//! let metric = Metric { name: "cpu_power".into(), value: 42, unit: "u64".into(), source: "RAPL".into() };
//! let metrics: Metrics = vec![metric];
//! ```

pub(crate) mod iteration;
mod metric;
pub(crate) mod phase;
pub(crate) mod sensor_result;

pub use metric::{Metric, Metrics};
