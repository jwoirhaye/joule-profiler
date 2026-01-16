use anyhow::Result;

use crate::{
    core::{
        measurement::{MeasurementResult, PhaseMeasurementResult},
        sensor::Sensor,
    },
    util::time::get_timestamp,
};

pub trait ProfilerDisplayer {
    fn simple_single(&mut self, cmd: &[String], _result: &MeasurementResult) -> Result<()>;

    fn simple_iterations(&mut self, _cmd: &[String], _results: &[MeasurementResult]) -> Result<()> {
        anyhow::bail!("Simple iterations not implemented for this format");
    }

    fn phases_single(
        &mut self,
        _cmd: &[String],
        _token_pattern: &str,
        _result: &PhaseMeasurementResult,
    ) -> Result<()> {
        anyhow::bail!("Phases single not implemented for this format");
    }

    fn phases_iterations(
        &mut self,
        _cmd: &[String],
        _token_pattern: &str,
        _results: &[PhaseMeasurementResult],
    ) -> Result<()> {
        anyhow::bail!("Phases iterations not implemented for this format");
    }
}

pub trait ListSensorsDisplayer {
    fn list_sensors(&mut self, _sensors: &[Sensor]) -> Result<()> {
        anyhow::bail!("List sensors not implemented for this format");
    }
}

pub fn default_iterations_filename(ext: &str) -> String {
    format!("data{}.{}", get_timestamp(), ext)
}
