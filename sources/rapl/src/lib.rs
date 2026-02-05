//! Module `rapl` — Intel RAPL metric source.
//!
//! This module provides several implementations of [`MetricReader`] for
//! collecting energy metrics from Intel RAPL (Running Average Power Limit) domains.
//!
//! # Backends
//!
//! This module supports **two backends** for reading energy metrics:
//! - [`powercap`] — uses the Linux `powercap` interface for energy readings.
//! - [`perf`] — uses perf_event counters (`perf_event_open`) for RAPL domains.
//!
//! # Features
//!
//! - Discover available RAPL domains under a given path.
//! - Read instantaneous energy consumption snapshots.
//! - Compute energy usage between consecutive snapshots.
//! - Provide sensors information for integration with the profiler.
//!
//! # Errors
//!
//! All RAPL operations return a [`RaplError`]. Possible errors include:
//! - [`RaplError::RaplNotAvailable`] - no RAPL domains found at the specified path.
//! - [`RaplError::InsufficientPermissions`] - requires elevated privileges to read powercap files.
//! - [`RaplError::UnsupportedOS`] - only Linux is supported.
//! - [`RaplError::RaplReadError`] or [`RaplError::InvalidRaplPath`] - problems reading counters or invalid paths.

use joule_profiler_core::unit::{MetricPrefix, MetricUnit, Unit};

mod domain_type;
mod error;
pub mod perf;
pub mod powercap;
mod snapshot;
mod util;

pub use error::RaplError;

/// Custom result type for Rapl
type Result<T> = std::result::Result<T, RaplError>;

const MICRO_JOULE_UNIT: MetricUnit = MetricUnit {
    prefix: MetricPrefix::Micro,
    unit: Unit::Joule,
};
