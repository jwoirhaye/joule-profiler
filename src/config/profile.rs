use std::collections::HashSet;

use crate::{
    core::displayer::ProfilerDisplayer,
    output::{OutputFormat, csv::CsvOutput, json::JsonOutput, terminal::TerminalOutput},
};

#[derive(Debug, Clone)]
pub struct ProfileConfig {
    pub iterations: usize,
    pub output_format: OutputFormat,
    pub jouleit_file: Option<String>,
    pub output_file: Option<String>,
    pub cmd: Vec<String>,
    pub sockets: Option<HashSet<u32>>,
    pub rapl_polling: Option<f64>,
    pub rapl_path: Option<String>,
    pub mode: Mode,
}

#[derive(Debug, Clone)]
pub enum Mode {
    SimpleMode,
    PhaseMode(PhasesConfig),
}

#[derive(Debug, Clone)]
pub struct PhasesConfig {
    pub token_pattern: String,
}

impl TryFrom<&ProfileConfig> for Box<dyn ProfilerDisplayer> {
    type Error = anyhow::Error;

    fn try_from(value: &ProfileConfig) -> Result<Self, Self::Error> {
        let displayer: Box<dyn ProfilerDisplayer> = match value.output_format {
            OutputFormat::Terminal => Box::new(TerminalOutput),
            OutputFormat::Json => Box::new(JsonOutput::new(value.output_file.clone())?),
            OutputFormat::Csv => Box::new(CsvOutput::new(value.output_file.clone())?),
        };
        Ok(displayer)
    }
}
