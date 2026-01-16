use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::Result;
use enum_dispatch::enum_dispatch;
use log::error;

use crate::{
    config::{ListSensorsConfig, OutputFormat, ProfileConfig},
    measurement::{MeasurementResult, PhaseMeasurementResult},
    output::{csv::CsvOutput, json::JsonOutput, terminal::TerminalOutput},
    source::Sensor,
};

mod csv;
mod json;
mod terminal;

#[enum_dispatch]
pub enum Displayer {
    Terminal(TerminalOutput),
    Json(JsonOutput),
    Csv(CsvOutput),
}

impl TryFrom<&ProfileConfig> for Displayer {
    type Error = anyhow::Error;

    fn try_from(config: &ProfileConfig) -> Result<Self, Self::Error> {
        Displayer::new(&config.output_format, config.jouleit_file.as_ref())
    }
}

impl TryFrom<&ListSensorsConfig> for Displayer {
    type Error = anyhow::Error;

    fn try_from(config: &ListSensorsConfig) -> Result<Self, Self::Error> {
        Displayer::new(&config.output_format, None)
    }
}

impl Displayer {
    pub fn new(output_format: &OutputFormat, jouleit_file: Option<&String>) -> Result<Self> {
        Ok(match output_format {
            OutputFormat::Terminal => Self::Terminal(TerminalOutput),
            OutputFormat::Json => Self::Json(JsonOutput::new(jouleit_file.cloned())?),
            OutputFormat::Csv => Self::Csv(CsvOutput::new(jouleit_file.cloned())?),
        })
    }
}

#[enum_dispatch(Displayer)]
pub trait OutputFormatTrait {
    fn simple_single(&mut self, _config: &ProfileConfig, _result: &MeasurementResult)
    -> Result<()>;

    fn simple_iterations(
        &mut self,
        _config: &ProfileConfig,
        _results: &[MeasurementResult],
    ) -> Result<()> {
        error!("Simple iterations not implemented for this format");
        anyhow::bail!("Simple iterations not implemented for this format");
    }

    fn phases_single(
        &mut self,
        _config: &ProfileConfig,
        _result: &PhaseMeasurementResult,
    ) -> Result<()> {
        error!("Phases single not implemented for this format");
        anyhow::bail!("Phases single not implemented for this format");
    }

    fn phases_iterations(
        &mut self,
        _config: &ProfileConfig,
        _results: &[PhaseMeasurementResult],
    ) -> Result<()> {
        error!("Phases iterations not implemented for this format");
        anyhow::bail!("Phases iterations not implemented for this format");
    }

    fn list_sensors(&mut self, _config: &ListSensorsConfig, _sensors: &[Sensor]) -> Result<()> {
        error!("List sensors not implemented for this format");
        anyhow::bail!("List sensors not implemented for this format");
    }
}

fn default_iterations_filename(ext: &str) -> String {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_secs();
    format!("data{}.{}", ts, ext)
}
