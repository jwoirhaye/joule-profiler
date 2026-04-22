//! NVML (NVIDIA Management Library) energy profiling integration.
//!
//! This module provides energy consumption monitoring for NVIDIA GPUs using the NVML library.
//! It implements the `MetricReader` trait to collect energy metrics from GPU devices and
//! track energy usage over time.

use std::collections::{HashMap, HashSet};

use joule_profiler_core::{
    sensor::Sensors,
    source::MetricReader,
    types::{Metric, Metrics},
    unit::{MetricUnit, Unit, UnitPrefix},
};

use crate::{
    config::NvmlConfig,
    error::NvmlError,
    hardware::{NvmlHardware, NvmlWrapperHardware},
    snapshot::{NvmlSnapshot, Phase},
};

pub mod config;
mod error;
mod hardware;
mod snapshot;

const NVML_SOURCE_NAME: &str = "NVML";
const MILLI_JOULE_UNIT: MetricUnit = MetricUnit {
    prefix: UnitPrefix::Milli,
    unit: Unit::Joule,
};

/// Custom result type for NVML.
type Result<T> = std::result::Result<T, NvmlError>;

/// NVML-based energy profiler for NVIDIA GPUs.
///
/// This struct provides an interface to monitor energy consumption of NVIDIA GPUs using
/// the NVML library.
/// The NVML hardware can be changed for testing purposes, but the default adapter is the NVML one.
pub struct Nvml<H: NvmlHardware = NvmlWrapperHardware> {
    /// The hardware instance for interacting with the NVIDIA driver.
    hardware: H,

    /// Accumulated energy consumption since last retrieval.
    begin_snapshot: Option<NvmlSnapshot>,

    /// The most recent snapshot taken, used to compute deltas.
    last_snapshot: Option<NvmlSnapshot>,
}

impl Nvml {
    pub fn new(gpus_spec: Option<HashSet<u32>>, exit_on_device_failure: bool) -> Result<Self> {
        Ok(Self {
            hardware: NvmlWrapperHardware::new(gpus_spec, exit_on_device_failure)?,
            begin_snapshot: None,
            last_snapshot: None,
        })
    }
}

impl<H: NvmlHardware + 'static> MetricReader for Nvml<H> {
    type Type = Phase;

    type Error = NvmlError;

    type Config = NvmlConfig;

    async fn measure(&mut self) -> Result<()> {
        let new_snapshot = self.hardware.read_snapshot()?;
        if self.begin_snapshot.is_none() {
            self.begin_snapshot = Some(new_snapshot);
        } else {
            self.last_snapshot = Some(new_snapshot);
        }
        Ok(())
    }

    async fn retrieve(&mut self) -> Result<Self::Type> {
        if let Some(begin) = self.begin_snapshot.take()
            && let Some(end) = self.last_snapshot.take()
        {
            self.begin_snapshot = Some(end.clone());
            Ok(Phase { begin, end })
        } else {
            Err(NvmlError::NotEnoughSamples)
        }
    }

    fn get_sensors(&self) -> Result<Sensors> {
        self.hardware.get_sensors()
    }

    fn to_metrics(&self, result: Self::Type) -> Result<Metrics> {
        let diff = Self::compute_energy_diff(&result.end, &result.begin)?;
        Ok(diff
            .gpus_energy
            .into_iter()
            .map(|(device_index, energy)| Metric {
                name: format!("GPU-{device_index}"),
                value: energy,
                unit: MILLI_JOULE_UNIT,
                source: NVML_SOURCE_NAME.to_string(),
            })
            .collect())
    }

    fn get_name() -> &'static str {
        NVML_SOURCE_NAME
    }

    fn from_config(config: Self::Config) -> Result<Self> {
        let gpus_spec = config
            .gpus_spec
            .map(|spec| spec.into_iter().collect::<HashSet<u32>>());
        Ok(Self {
            hardware: H::new(gpus_spec, config.exit_on_device_failure)?,
            begin_snapshot: None,
            last_snapshot: None,
        })
    }

    fn get_id() -> &'static str {
        "nvml"
    }
}

impl<H: NvmlHardware + 'static> Nvml<H> {
    /// Computes the energy consumption difference between two snapshots.
    ///
    /// This calculates the delta in energy consumption for each GPU device between
    /// the old and new snapshots. Uses wrapping subtraction to handle counter wraparound
    /// correctly (even so it will never occur in theory).
    fn compute_energy_diff(
        end_snapshot: &NvmlSnapshot,
        begin_snapshot: &NvmlSnapshot,
    ) -> Result<NvmlSnapshot> {
        let mut gpus_energy = HashMap::new();
        for (device_index, old_energy) in &begin_snapshot.gpus_energy {
            let new_energy =
                end_snapshot
                    .gpus_energy
                    .get(device_index)
                    .ok_or(NvmlError::UnknownMetricError(format!(
                        "Device {device_index} unknown"
                    )))?;
            let diff = new_energy.wrapping_sub(*old_energy);
            gpus_energy.insert(*device_index, diff);
        }
        Ok(NvmlSnapshot { gpus_energy })
    }
}

#[cfg(test)]
mod tests {
    use joule_profiler_core::sensor::Sensor;
    use mockall::mock;

    use super::*;
    use crate::snapshot::NvmlSnapshot;

    mock! {
        pub NvmlHardware {}

        impl NvmlHardware for NvmlHardware {
            fn new(gpus_spec: Option<HashSet<u32>>, exit_on_device_failure: bool) -> Result<MockNvmlHardware>
            where
                MockNvmlHardware: Sized;

            fn read_snapshot(&self) -> Result<NvmlSnapshot>;
            fn get_sensors(&self) -> Result<Sensors>;
        }
    }

    fn snapshot(entries: Vec<(u32, u64)>) -> NvmlSnapshot {
        NvmlSnapshot {
            gpus_energy: entries.into_iter().collect(),
        }
    }

    fn sensors(count: u32) -> Sensors {
        (0..count)
            .map(|i| Sensor {
                name: format!("GPU-{i}"),
                unit: MILLI_JOULE_UNIT,
                source: NVML_SOURCE_NAME.to_string(),
            })
            .collect()
    }

    fn nvml_with_hardware(hardware: MockNvmlHardware) -> Nvml<MockNvmlHardware> {
        Nvml {
            hardware,
            begin_snapshot: None,
            last_snapshot: None,
        }
    }

    #[test]
    fn diff_compute_right_values() {
        let begin = snapshot(vec![(0, 100), (1, 200)]);
        let end = snapshot(vec![(0, 150), (1, 300)]);
        let diff = Nvml::<MockNvmlHardware>::compute_energy_diff(&end, &begin).unwrap();
        assert_eq!(diff.gpus_energy[&0], 50);
        assert_eq!(diff.gpus_energy[&1], 100);
    }

    #[test]
    fn diff_wraps_on_counter_overflow() {
        let begin = snapshot(vec![(0, u64::MAX - 5)]);
        let end = snapshot(vec![(0, 10)]);
        let diff = Nvml::<MockNvmlHardware>::compute_energy_diff(&end, &begin).unwrap();
        assert_eq!(diff.gpus_energy[&0], 16);
    }

    #[test]
    fn diff_device_missing_in_end_returns_error() {
        let begin = snapshot(vec![(0, 100), (1, 200)]);
        let end = snapshot(vec![(0, 150)]);
        let result = Nvml::<MockNvmlHardware>::compute_energy_diff(&end, &begin);
        assert!(matches!(result, Err(NvmlError::UnknownMetricError(_))));
    }

    #[tokio::test]
    async fn measure_stores_begin_snapshot() {
        let mut hardware = MockNvmlHardware::default();
        hardware
            .expect_read_snapshot()
            .returning(|| Ok(snapshot(vec![(0, 100)])));

        let mut nvml = nvml_with_hardware(hardware);
        nvml.measure().await.unwrap();
        assert!(nvml.begin_snapshot.is_some());
        assert!(nvml.last_snapshot.is_none());
    }

    #[tokio::test]
    async fn measure_twice_stores_last_snapshot() {
        let mut hardware = MockNvmlHardware::default();
        let mut read_snapshot_call_count = 0u64;
        hardware.expect_read_snapshot().returning(move || {
            read_snapshot_call_count += 1;
            Ok(snapshot(vec![(0, read_snapshot_call_count * 100)]))
        });

        let mut nvml = nvml_with_hardware(hardware);
        nvml.measure().await.unwrap();
        nvml.measure().await.unwrap();

        assert!(nvml.begin_snapshot.is_some());
        assert!(nvml.last_snapshot.is_some());
    }

    #[tokio::test]
    async fn retrieve_without_enough_snapshots_returns_error() {
        let mut hardware = MockNvmlHardware::default();
        hardware
            .expect_read_snapshot()
            .returning(|| Ok(snapshot(vec![(0, 100)])));

        let mut nvml = nvml_with_hardware(hardware);
        nvml.measure().await.unwrap();

        assert!(matches!(
            nvml.retrieve().await,
            Err(NvmlError::NotEnoughSamples)
        ));
    }

    #[tokio::test]
    async fn retrieve_returns_correct_phase() {
        let mut hardware = MockNvmlHardware::default();
        let mut read_snapshot_call_count = 0;

        hardware.expect_read_snapshot().returning(move || {
            read_snapshot_call_count += 1;
            Ok(snapshot(vec![(0, read_snapshot_call_count * 100)]))
        });
        let mut nvml = nvml_with_hardware(hardware);

        nvml.measure().await.unwrap();
        nvml.measure().await.unwrap();
        let phase = nvml.retrieve().await.unwrap();

        assert_eq!(phase.begin.gpus_energy[&0], 100);
        assert_eq!(phase.end.gpus_energy[&0], 200);
    }

    #[tokio::test]
    async fn retrieve_replace_begin_snapshot_with_end() {
        let mut hardware = MockNvmlHardware::default();
        let mut read_snapshot_call_count = 0u64;

        hardware.expect_read_snapshot().returning(move || {
            read_snapshot_call_count += 1;
            Ok(snapshot(vec![(0, read_snapshot_call_count * 100)]))
        });

        let mut nvml = nvml_with_hardware(hardware);
        nvml.measure().await.unwrap();
        nvml.measure().await.unwrap();
        nvml.retrieve().await.unwrap();

        assert_eq!(nvml.begin_snapshot.as_ref().unwrap().gpus_energy[&0], 200);
        assert!(nvml.last_snapshot.is_none());
    }

    #[tokio::test]
    async fn to_metrics_returns_correct_values() {
        let mut hardware = MockNvmlHardware::default();
        let mut read_snapshot_call_count = 0;

        hardware.expect_read_snapshot().returning(move || {
            read_snapshot_call_count += 1;
            Ok(match read_snapshot_call_count {
                1 => snapshot(vec![(0, 0), (1, 0)]),
                _ => snapshot(vec![(0, 100), (1, 200)]),
            })
        });

        let mut nvml = nvml_with_hardware(hardware);
        nvml.measure().await.unwrap();
        nvml.measure().await.unwrap();
        let phase = nvml.retrieve().await.unwrap();
        let mut metrics = nvml.to_metrics(phase).unwrap();
        metrics.sort_by_key(|m| m.name.clone());

        assert_eq!(metrics.len(), 2);
        assert_eq!(metrics[0].name, "GPU-0");
        assert_eq!(metrics[0].value, 100);
        assert_eq!(metrics[0].unit, MILLI_JOULE_UNIT);
        assert_eq!(metrics[1].name, "GPU-1");
        assert_eq!(metrics[1].value, 200);
        assert_eq!(metrics[1].unit, MILLI_JOULE_UNIT);
    }

    #[test]
    fn get_sensors_returns_one_sensor_per_device() {
        let mut hardware = MockNvmlHardware::default();
        hardware.expect_get_sensors().returning(|| Ok(sensors(2)));

        let nvml = nvml_with_hardware(hardware);
        let sensors = nvml.get_sensors().unwrap();
        assert_eq!(sensors.len(), 2);
        assert_eq!(sensors[0].name, "GPU-0");
        assert_eq!(sensors[1].name, "GPU-1");
    }
}
