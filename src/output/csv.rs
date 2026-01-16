use std::collections::HashSet;
use std::fs::File;
use std::io::Write;

use anyhow::Result;
use log::{debug, info, trace, warn};

use crate::config::{ListSensorsConfig, ProfileConfig};
use crate::measurement::{MeasurementResult, PhaseMeasurementResult, PhaseResult};
use crate::output::OutputFormatTrait;
use crate::source::Sensor;
use crate::util::file::{create_file_with_user_permissions, get_absolute_path};

use super::default_iterations_filename;

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

impl OutputFormatTrait for CsvOutput {
    fn simple_single(&mut self, config: &ProfileConfig, result: &MeasurementResult) -> Result<()> {
        debug!("Formatting simple single measurement for CSV");
        let keys: Vec<&String> = result.metrics.iter().map(|metric| &metric.name).collect();

        self.write_header(&keys, false, false)?;
        self.write_row(&config.cmd, result, None)?;

        self.finalize();
        Ok(())
    }

    fn simple_iterations(
        &mut self,
        config: &ProfileConfig,
        results: &[MeasurementResult],
    ) -> Result<()> {
        info!("Formatting {} simple iterations for CSV", results.len());

        if results.is_empty() {
            warn!("No iterations to write to CSV");
            return Ok(());
        }

        let first = &results[0];
        let keys: Vec<&String> = first.extract_keys();

        debug!("CSV will contain {} metrics", first.metrics.len());

        self.write_header(&keys, true, false)?;

        for (idx, res) in results.iter().enumerate() {
            self.write_row(&config.cmd, &res, Some(idx))?;
        }

        self.finalize();
        Ok(())
    }

    fn phases_single(
        &mut self,
        config: &ProfileConfig,
        result: &PhaseMeasurementResult,
    ) -> Result<()> {
        debug!(
            "Formatting phases single measurement for CSV ({} phases)",
            result.phases.len()
        );

        if result.phases.is_empty() {
            warn!("No phases to write to CSV");
            return Ok(());
        }

        let keys: Vec<&String> = result.extract_keys();
        self.write_header(&keys, false, true)?;

        for phase in &result.phases {
            trace!("Writing phase: {}", phase.name);

            let phase_data = PhaseRowData::new(
                &phase.name,
                phase.start_token.as_deref(),
                phase.end_token.as_deref(),
                phase.start_line,
                phase.end_line,
            );

            self.write_row_phase(&config.cmd, &phase, None, &phase_data)?;
        }

        self.finalize();
        Ok(())
    }

    fn phases_iterations(
        &mut self,
        config: &ProfileConfig,
        results: &[PhaseMeasurementResult],
    ) -> Result<()> {
        info!("Formatting {} phase iterations for CSV", results.len());

        if results.is_empty() {
            warn!("No phase iterations to write to CSV");
            return Ok(());
        }

        let first_result = &results[0];

        if first_result.phases.is_empty() {
            warn!("No phases to write to CSV");
            return Ok(());
        }

        let mut keys: HashSet<&String> = HashSet::new();
        for phase_result in results {
            keys.extend(phase_result.extract_keys());
        }
        let keys_vec: Vec<&String> = keys.into_iter().collect();

        debug!("CSV will contain {} metrics", keys_vec.len());

        self.write_header(&keys_vec, true, true)?;

        for (idx, iteration_results) in results.iter().enumerate() {
            for phase in &iteration_results.phases {
                trace!("Writing iteration {} phase: {}", idx + 1, phase.name);

                let phase_data = PhaseRowData::new(
                    &phase.name,
                    phase.start_token.as_deref(),
                    phase.end_token.as_deref(),
                    phase.start_line,
                    phase.end_line,
                );

                self.write_row_phase(&config.cmd, &phase, Some(idx), &phase_data)?;
            }
        }

        self.finalize();
        Ok(())
    }

    fn list_sensors(&mut self, _config: &ListSensorsConfig, sensors: &[Sensor]) -> Result<()> {
        write!(self.file, "sensor;unit;source")?;
        for sensor in sensors {
            write!(
                self.file,
                "{};{};{}",
                sensor.name, sensor.unit, sensor.source
            )?;
        }
        Ok(())
    }
}

impl CsvOutput {
    pub fn new(output_file: Option<String>) -> Result<Self> {
        let filename = output_file
            .clone()
            .unwrap_or(default_iterations_filename("csv"));

        let absolute_path = get_absolute_path(&filename)?;
        info!("Creating CSV output file: {}", absolute_path);

        let file = create_file_with_user_permissions(&absolute_path)?;

        Ok(Self {
            file,
            filename: absolute_path,
        })
    }

    fn write_header(
        &mut self,
        keys: &[&String],
        include_iteration: bool,
        include_phase: bool,
    ) -> Result<()> {
        trace!("Writing CSV header with {} metrics", keys.len());

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

        for key in keys {
            write!(self.file, "{};", key)?;
        }
        writeln!(
            self.file,
            "duration_ms;measure_count;measure_delta;exit_code"
        )?;

        debug!("CSV header written");
        Ok(())
    }

    fn write_row(
        &mut self,
        command: &[String],
        result: &MeasurementResult,
        iteration: Option<usize>,
    ) -> Result<()> {
        write!(self.file, "'{}';", command.join(" "))?;

        if let Some(idx) = iteration {
            trace!("Writing CSV row for iteration {}", idx);
            write!(self.file, "{};", idx)?;
        }

        for metric in &result.metrics {
            write!(self.file, "{};", metric.value)?;
        }

        write!(
            self.file,
            "{};{};{};{};",
            result.duration_ms, result.measure_count, result.measure_delta, result.exit_code
        )?;

        Ok(())
    }

    fn write_row_phase(
        &mut self,
        command: &[String],
        result: &PhaseResult,
        iteration: Option<usize>,
        phase: &PhaseRowData,
    ) -> Result<()> {
        write!(self.file, "'{}';", command.join(" "))?;

        if let Some(idx) = iteration {
            trace!("Writing CSV row for iteration {}", idx);
            write!(self.file, "{};", idx)?;
        }

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

        for metric in &result.metrics {
            write!(self.file, "{};", metric.value)?;
        }

        write!(self.file, "{};", result.duration_ms)?;

        Ok(())
    }

    fn finalize(&self) {
        println!("✔ CSV written to: {}", self.filename);
        info!("CSV output saved to: {}", self.filename);
    }
}
