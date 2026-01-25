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
//! use joule_profiler_core::sensor::{Sensor, Sensors};
//!
//! // Create a single sensor
//! let cpu_sensor = Sensor::new(
//!     "CORE-0".to_string(),
//!     "µJ".to_string(),
//!     "powercap".to_string(),
//! );
//!
//! // Collect sensors into a vector
//! let sensors: Sensors = vec![cpu_sensor];
//! assert_eq!(sensors.len(), 1);
//! ```

use serde::Serialize;

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
/// use joule_profiler_core::sensor::Sensor;
///
/// let sensor = Sensor {
///     name: "CORE-0".to_string(),
///     unit: "µJ".to_string(),
///     source: "powercap".to_string(),
/// };
/// assert_eq!(sensor.name, "CORE-0");
/// assert_eq!(sensor.unit, "µJ");
/// assert_eq!(sensor.source, "powercap");
/// ```
#[derive(Debug, Serialize)]
pub struct Sensor {
    pub name: String,

    pub unit: String,

    pub source: String,
}

impl Sensor {
    /// Create a new sensor
    ///
    /// # Arguments
    ///
    /// - `name` (`String`) - The sensor name
    /// - `unit` (`String`) - Unit of measurement
    /// - `source` (`String`) - The origin of the sensor
    ///
    /// # Returns
    ///
    /// - 'Self' - The sensor
    ///
    /// # Examples
    ///
    /// ```
    /// use joule_profiler_core::sensor::Sensor;
    ///
    /// let sensor = Sensor::new("CPU Energy".to_string(), "Joule".to_string(), "RAPL".to_string());
    /// assert_eq!(sensor.name, "CPU Energy");
    /// ```
    pub fn new(name: String, unit: String, source: String) -> Self {
        Self { name, unit, source }
    }
}

/// A collection of sensors.
pub type Sensors = Vec<Sensor>;
