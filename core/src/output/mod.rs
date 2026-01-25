//! Output formats for JouleProfiler.
//!
//! This module defines the supported output formats and provides utilities
//! for selecting and displaying metrics collected by JouleProfiler.
//! It includes built-in formats for terminal display, JSON export, and CSV export.
//!
//! # Overview
//!
//! - [`OutputFormat`] — Enum representing the available output formats.
//! - `csv`, `json`, `terminal` — Submodules implementing the actual display logic for default output formats.

use std::fmt::{Display, Formatter, Result};

mod csv;
mod json;
mod terminal;

pub use csv::CsvOutput;
pub use json::JsonOutput;
pub use terminal::TerminalOutput;

/// Represents the supported output formats for JouleProfiler.
///
/// This enum defines how metrics are displayed or exported. It is used
/// by the profiler to select the appropriate output method based on
/// user preferences or CLI flags.
/// The output format is used to instanciate a [`crate::displayer::Displayer`].
///
/// # Variants
///
/// - `Terminal` — Display metrics directly in the terminal (default).
/// - `Json` — Export metrics as JSON for easy parsing or integration.
/// - `Csv` — Export metrics in CSV format for spreadsheets or analysis.
#[derive(Debug, Clone, Copy, Default)]
pub enum OutputFormat {
    #[default]
    Terminal,

    Json,

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
