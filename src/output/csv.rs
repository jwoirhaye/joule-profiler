use std::fs::File;
use std::io::Write;

use anyhow::Result;
use log::{debug, info, trace, warn};

use crate::config::Config;
use crate::measure::{MeasurementResult, PhasesResult};

use super::{OutputFormat, default_iterations_filename, get_absolute_path};

/// Data for a phase row in CSV output
struct PhaseRowData<'a> {
    name: &'a str,
    start_token: Option<&'a str>,
    end_token: Option<&'a str>,
    start_line: Option<usize>,
    end_line: Option<usize>,
}

impl<'a> PhaseRowData<'a> {
    fn new(
        name: &'a str,
        start_token: Option<&'a str>,
        end_token: Option<&'a str>,
        start_line: Option<usize>,
        end_line: Option<usize>,
    ) -> Self {
        Self {
            name,
            start_token,
            end_token,
            start_line,
            end_line,
        }
    }
}

pub struct CsvOutput {
    file: File,
    filename: String,
}

impl CsvOutput {
    pub fn new(config: &Config) -> Result<Self> {
        let filename = config
            .jouleit_file
            .clone()
            .unwrap_or_else(|| default_iterations_filename("csv"));

        let absolute_path = get_absolute_path(&filename)?;
        info!("Creating CSV output file: {}", absolute_path);

        let file = File::create(&filename)?;

        Ok(Self {
            file,
            filename: absolute_path,
        })
    }

    fn write_header(
        &mut self,
        keys: &[String],
        include_iteration: bool,
        include_phase: bool,
    ) -> Result<()> {
        trace!("Writing CSV header with {} energy domains", keys.len());

        write!(self.file, "command;")?;

        if include_iteration {
            write!(self.file, "iteration;")?;
        }

        if include_phase {
            write!(
                self.file,
                "phase_name;start_token;end_token;start_line;end_line;"
            )?;
        }

        for k in keys {
            write!(self.file, "{}_uj;", k)?;
        }
        writeln!(self.file, "duration_ms;exit_code")?;

        debug!("CSV header written");
        Ok(())
    }

    fn write_row(
        &mut self,
        command: &[String],
        res: &MeasurementResult,
        keys: &[String],
        iteration: Option<usize>,
        phase_data: Option<&PhaseRowData>,
    ) -> Result<()> {
        write!(self.file, "'{}';", command.join(" "))?;

        if let Some(idx) = iteration {
            trace!("Writing CSV row for iteration {}", idx);
            write!(self.file, "{};", idx)?;
        }

        if let Some(phase) = phase_data {
            write!(self.file, "{};", phase.name)?;
            write!(self.file, "{};", phase.start_token.unwrap_or(""))?;
            write!(self.file, "{};", phase.end_token.unwrap_or(""))?;
            write!(
                self.file,
                "{};",
                phase.start_line.map(|l| l.to_string()).unwrap_or_default()
            )?;
            write!(
                self.file,
                "{};",
                phase.end_line.map(|l| l.to_string()).unwrap_or_default()
            )?;
        }

        for k in keys {
            let v = res.energy_uj.get(k).copied().unwrap_or(0);
            write!(self.file, "{};", v)?;
        }
        writeln!(self.file, "{};{}", res.duration_ms, res.exit_code)?;

        Ok(())
    }

    fn finalize(&self) {
        println!("✔ CSV written to: {}", self.filename);
        info!("CSV output saved to: {}", self.filename);
    }
}

impl OutputFormat for CsvOutput {
    fn simple_single(&mut self, config: &Config, res: &MeasurementResult) -> Result<()> {
        debug!("Formatting simple single measurement for CSV");

        let mut keys: Vec<_> = res.energy_uj.keys().cloned().collect();
        keys.sort();

        debug!("CSV will contain {} energy domains", keys.len());

        self.write_header(&keys, false, false)?;

        self.write_row(&config.cmd, res, &keys, None, None)?;

        self.finalize();
        Ok(())
    }

    fn simple_iterations(
        &mut self,
        config: &Config,
        results: &[(usize, MeasurementResult)],
    ) -> Result<()> {
        info!("Formatting {} simple iterations for CSV", results.len());

        if results.is_empty() {
            warn!("No iterations to write to CSV");
            return Ok(());
        }

        let (_, first) = &results[0];
        let mut keys: Vec<_> = first.energy_uj.keys().cloned().collect();
        keys.sort();

        debug!("CSV will contain {} energy domains", keys.len());

        self.write_header(&keys, true, false)?;

        for (idx, res) in results {
            self.write_row(&config.cmd, res, &keys, Some(*idx), None)?;
        }

        self.finalize();
        Ok(())
    }

    fn phases_single(&mut self, config: &Config, phases: &PhasesResult) -> Result<()> {
        debug!(
            "Formatting phases single measurement for CSV ({} phases)",
            phases.phases.len()
        );

        if phases.phases.is_empty() {
            warn!("No phases to write to CSV");
            return Ok(());
        }

        let mut all_keys = std::collections::HashSet::new();
        for phase in &phases.phases {
            for key in phase.result.energy_uj.keys() {
                all_keys.insert(key.clone());
            }
        }
        let mut keys: Vec<_> = all_keys.into_iter().collect();
        keys.sort();

        debug!("CSV will contain {} energy domains", keys.len());

        self.write_header(&keys, false, true)?;

        for phase in &phases.phases {
            trace!("Writing phase: {}", phase.name);

            let phase_data = PhaseRowData::new(
                &phase.name,
                phase.start_token.as_deref(),
                phase.end_token.as_deref(),
                phase.start_line,
                phase.end_line,
            );

            self.write_row(&config.cmd, &phase.result, &keys, None, Some(&phase_data))?;
        }

        self.finalize();
        Ok(())
    }

    fn phases_iterations(
        &mut self,
        config: &Config,
        results: &[(usize, PhasesResult)],
    ) -> Result<()> {
        info!("Formatting {} phase iterations for CSV", results.len());

        if results.is_empty() {
            warn!("No phase iterations to write to CSV");
            return Ok(());
        }

        let mut all_keys = std::collections::HashSet::new();

        if let Some((_, first_result)) = results.first() {
            for phase in &first_result.phases {
                for key in phase.result.energy_uj.keys() {
                    all_keys.insert(key.clone());
                }
            }
        }

        let mut keys: Vec<_> = all_keys.into_iter().collect();
        keys.sort();

        debug!("CSV will contain {} energy domains", keys.len());

        self.write_header(&keys, true, true)?;

        for (idx, phases_result) in results {
            for phase in &phases_result.phases {
                trace!("Writing iteration {} phase: {}", idx, phase.name);

                let phase_data = PhaseRowData::new(
                    &phase.name,
                    phase.start_token.as_deref(),
                    phase.end_token.as_deref(),
                    phase.start_line,
                    phase.end_line,
                );

                self.write_row(
                    &config.cmd,
                    &phase.result,
                    &keys,
                    Some(*idx),
                    Some(&phase_data),
                )?;
            }
        }

        self.finalize();
        Ok(())
    }
}
