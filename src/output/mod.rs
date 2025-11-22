use anyhow::Result;
use log::{debug, trace, warn};
use std::path::PathBuf;

use crate::config::Config;
use crate::measure::{MeasurementResult, PhasesResult};

pub mod csv;
pub mod json;
pub mod terminal;

pub use json::JsonOutput;
pub use terminal::TerminalOutput;

pub trait OutputFormat {
    fn simple_single(&mut self, res: &MeasurementResult) -> Result<()>;

    fn simple_iterations(
        &mut self,
        _config: &Config,
        _results: &[(usize, MeasurementResult)],
    ) -> Result<()> {
        warn!("Simple iterations not implemented for this output format");
        anyhow::bail!("Simple iterations not implemented for this format");
    }

    fn phases_single(&mut self, _config: &Config, _phases: &PhasesResult) -> Result<()> {
        warn!("Phases single not implemented for this output format");
        anyhow::bail!("Phases single not implemented for this format");
    }

    fn phases_iterations(
        &mut self,
        _config: &Config,
        _results: &[(usize, PhasesResult)],
    ) -> Result<()> {
        warn!("Phases iterations not implemented for this output format");
        anyhow::bail!("Phases iterations not implemented for this format");
    }
}

pub(crate) fn default_iterations_filename(ext: &str) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    trace!("Generating default filename with extension: {}", ext);

    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|e| {
            warn!("System time is before UNIX_EPOCH: {}, using 0", e);
            std::time::Duration::from_secs(0)
        })
        .as_secs();

    let filename = format!("data{}.{}", ts, ext);
    debug!("Generated default filename: {}", filename);

    filename
}

pub(crate) fn get_absolute_path(filename: &str) -> Result<String> {
    let path = PathBuf::from(filename);
    let absolute_path = if path.is_absolute() {
        path
    } else {
        std::env::current_dir()?.join(&path)
    };

    Ok(absolute_path.display().to_string())
}
