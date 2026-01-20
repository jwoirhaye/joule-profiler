use std::fmt::{Display, Formatter, Result};

pub mod csv;
pub mod json;
pub mod terminal;

/// Supported output formats
#[derive(Debug, Clone, Copy)]
pub enum OutputFormat {
    /// Output to terminal
    Terminal,

    /// Output as JSON
    Json,

    /// Output as CSV
    Csv,
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

impl Default for OutputFormat {
    fn default() -> Self {
        Self::Terminal
    }
}

/// Determine output format from flags
pub fn output_format(json: bool, csv: bool) -> OutputFormat {
    if json {
        OutputFormat::Json
    } else if csv {
        OutputFormat::Csv
    } else {
        OutputFormat::Terminal
    }
}

