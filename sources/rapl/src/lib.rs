//! Module `rapl` — Intel RAPL metric source.
//!
//! This module provides an implementation of a [`MetricReader`] for
//! collecting energy metrics from Intel RAPL (Running Average Power Limit) domains.
//!
//! The `Rapl` struct manages RAPL domains, reads energy counters,
//! and optionally supports periodic polling for continuous measurement.
//!
//! # Features
//!
//! - Discover available RAPL domains under a given path.
//! - Read instantaneous energy consumption snapshots.
//! - Compute energy usage between consecutive snapshots.
//! - Provide sensors information for integration with the profiler.
//! - Optional async scheduler for periodic measurement.
//!
//! # Usage
//!
//! ```no_run
//! use source_rapl::Rapl;
//! use joule_profiler_core::source::MetricReader;
//!
//! #[tokio::main]
//! async fn main() {
//!     // Initialize a RAPL reader (no polling, monitoring all sockets)
//!     let mut rapl = Rapl::from_default().unwrap();
//!
//!     // Measure and update internal counters
//!     rapl.measure().await.unwrap();
//!
//!     // Retrieve available sensors
//!     let sensors = rapl.get_sensors().unwrap();
//!
//!     // Retrieve collected counters
//!     let counters = rapl.retrieve().await.unwrap();
//! }
//! ```
//!
//! # Errors
//!
//! All RAPL operations return a [`RaplError`]. Possible errors include:
//! - [`RaplError::RaplNotAvailable`] - no RAPL domains found at the specified path.
//! - [`RaplError::InsufficientPermissions`] - requires elevated privileges to read powercap files.
//! - [`RaplError::UnsupportedOS`] - only Linux is supported.
//! - [`RaplError::RaplReadError`] or [`RaplError::InvalidRaplPath`] - problems reading counters or invalid paths.

use joule_profiler_core::unit::{MetricPrefix, MetricUnit, Unit};

use crate::error::RaplError;

mod domain;
pub mod error;
pub mod perf;
pub mod powercap;

/// Custom result type for Rapl
type Result<T> = std::result::Result<T, RaplError>;

const MICRO_JOULE_UNIT: MetricUnit = MetricUnit {
    prefix: MetricPrefix::Micro,
    unit: Unit::Joule,
};
