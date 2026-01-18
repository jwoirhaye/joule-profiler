use anyhow::Result;

use crate::{
    config::Config,
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

    fn try_from(config: &Config) -> std::result::Result<Self, Self::Error> {
        let output_file = match &config.mode {
            crate::config::Command::Profile(profile_config) => profile_config.output_file.clone(),
            crate::config::Command::ListSensors(_) => None,
        };

        let displayer: Box<dyn Displayer> = match config.output_format {
            OutputFormat::Terminal => Box::new(TerminalOutput),
            OutputFormat::Json => Box::new(JsonOutput::new(output_file)?),
            OutputFormat::Csv => Box::new(CsvOutput::new(output_file)?),
        };

        Ok(displayer)
    }
}

pub fn default_iterations_filename(ext: &str) -> String {
    format!("data{}.{}", get_timestamp(), ext)
}
