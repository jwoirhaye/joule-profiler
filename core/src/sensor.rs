//! Representation of measurable sensors.
//!
//! This module defines the structures used to describe sensors collected
//! by JouleProfiler. Sensors are associated with metric sources and are
//! used to represent individual measurements.
//!
//! # Structures
//!
//! - [`Sensor`] — Represents a single measurable sensor, with a name, unit, and source.
//! - [`Sensors`] — A collection of [`Sensor`] objects.
//!
//! # Examples
//!
//! ```no_run
//! use joule_profiler_core::{
//!     sensor::{Sensor, Sensors},
//!     unit::{MetricUnit, UnitPrefix, Unit},
//! };
//!
//! let micro_joule_unit: MetricUnit = MetricUnit {
//!     prefix: UnitPrefix::Micro,
//!     unit: Unit::Joule,
//! };
//!
//! // Create a single sensor
//! let cpu_sensor = Sensor {
//!     name: "CORE-0".to_string(),
//!     unit: micro_joule_unit,
//!     source: "powercap".to_string(),
//! };
//!
//! // Collect sensors into a vector
//! let sensors: Sensors = vec![cpu_sensor];
//! assert_eq!(sensors.len(), 1);
//! ```

use serde::Serialize;

use crate::unit::MetricUnit;

/// Represents a measurable sensor.
///
/// A sensor corresponds to a metric collected from a source. Each sensor
/// has a name, a unit of measurement, and an origin indicating the source
/// providing this metric.
///
/// # Fields
///
/// - `name` (`String`) - The human-readable name of the sensor (e.g., `"CORE-0"`).
/// - `unit` (`String`) - The unit of measurement for the sensor (e.g., `"µJ"`).
/// - `source` (`String`) - The origin of the sensor (e.g., `"powercap"`).
///
/// # Examples
///
/// ```no_run
/// use joule_profiler_core::{
///     sensor::Sensor,
///     unit::{MetricUnit, UnitPrefix, Unit},
/// };
///
/// let micro_joule_unit: MetricUnit = MetricUnit {
///     prefix: UnitPrefix::Micro,
///     unit: Unit::Joule,
/// };
///
/// let sensor = Sensor {
///     name: "CORE-0".to_string(),
///     unit: micro_joule_unit,
///     source: "powercap".to_string(),
/// };
/// assert_eq!(sensor.name, "CORE-0");
/// assert_eq!(sensor.unit.to_string(), "µJ");
/// assert_eq!(sensor.source, "powercap");
/// ```
#[derive(Debug, Serialize)]
pub struct Sensor {
    pub name: String,

    pub unit: MetricUnit,

    pub source: String,
}

/// A collection of sensors.
pub type Sensors = Vec<Sensor>;
