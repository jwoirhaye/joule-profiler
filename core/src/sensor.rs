//! Representation of measurable sensors.
//!
//! This module defines the structures used to describe sensors collected
//! by JouleProfiler. Sensors are associated with metric sources and are
//! used to represent individual measurements.

use serde::Serialize;

use crate::unit::MetricUnit;

/// Represents a measurable sensor.
///
/// A sensor corresponds to a metric collected from a source.
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
    /// The name of the sensor.
    pub name: String,

    /// The standard international unit associated to this sensor.
    pub unit: MetricUnit,

    /// The metric source associated to the sensor.
    pub source: String,
}

/// A collection of sensors.
pub type Sensors = Vec<Sensor>;
