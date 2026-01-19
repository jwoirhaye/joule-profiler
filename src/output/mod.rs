use std::fmt::{Display, Formatter, Result};

pub mod csv;
pub mod json;
pub mod terminal;

#[derive(Debug, Clone, Copy)]
pub enum OutputFormat {
    Terminal,
    Json,
    Csv,
}

pub fn output_format(json: bool, csv: bool) -> OutputFormat {
    if json {
        OutputFormat::Json
    } else if csv {
        OutputFormat::Csv
    } else {
        OutputFormat::Terminal
    }
}

impl Display for OutputFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.write_str(match self {
            OutputFormat::Terminal => "Terminal",
            OutputFormat::Json => "Json",
            OutputFormat::Csv => "CSV",
        })?;
        Ok(())
    }
}
