use anyhow::Result;

use crate::{
    config::{Command, Config},
    core::{
        measurement::{MeasurementResult, PhaseMeasurementResult},
        sensor::Sensor,
    },
    output::{OutputFormat, csv::CsvOutput, json::JsonOutput, terminal::TerminalOutput},
    util::time::get_timestamp,
};

pub trait Displayer {
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

    fn list_sensors(&mut self, _sensors: &[Sensor]) -> Result<()> {
        anyhow::bail!("List sensors not implemented for this format");
    }
}

impl TryFrom<&Config> for Box<dyn Displayer> {
    type Error = anyhow::Error;

    fn try_from(config: &Config) -> Result<Self, Self::Error> {
        let output_file = match &config.mode {
            Command::Profile(profile_config) => profile_config.output_file.clone(),
            Command::ListSensors(_) => None,
        };

        let displayer = match config.output_format {
            OutputFormat::Terminal => TerminalOutput.into(),
            OutputFormat::Json => JsonOutput::new(output_file)?.into(),
            OutputFormat::Csv => CsvOutput::try_new(output_file)?.into(),
        };

        Ok(displayer)
    }
}

impl<T: Displayer + 'static> From<T> for Box<dyn Displayer> {
    fn from(displayer: T) -> Self {
        Box::new(displayer)
    }
}

pub fn default_iterations_filename(ext: &str) -> String {
    format!("data{}.{}", get_timestamp(), ext)
}
