use std::collections::HashSet;
use std::fs::File;
use std::io::Write;

use anyhow::Result;

use crate::core::displayer::{
    ListSensorsDisplayer, ProfilerDisplayer, default_iterations_filename,
};
use crate::core::measurement::{MeasurementResult, PhaseMeasurementResult, PhaseResult};
use crate::core::sensor::Sensor;
use crate::util::file::{create_file_with_user_permissions, get_absolute_path};

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

impl ProfilerDisplayer for CsvOutput {
    fn simple_single(&mut self, cmd: &[String], result: &MeasurementResult) -> Result<()> {
        let keys: Vec<&String> = result.metrics.iter().map(|metric| &metric.name).collect();

        self.write_header(&keys, false, false)?;
        self.write_row(cmd, result, None)?;

        self.finalize();
        Ok(())
    }

    fn simple_iterations(&mut self, cmd: &[String], results: &[MeasurementResult]) -> Result<()> {
        if results.is_empty() {
            return Ok(());
        }

        let first = &results[0];
        let keys: Vec<&String> = first.extract_keys();

        self.write_header(&keys, true, false)?;

        for (idx, result) in results.iter().enumerate() {
            self.write_row(cmd, result, Some(idx))?;
        }

        self.finalize();
        Ok(())
    }

    fn phases_single(
        &mut self,
        cmd: &[String],
        _token_pattern: &str,
        result: &PhaseMeasurementResult,
    ) -> Result<()> {
        if result.phases.is_empty() {
            return Ok(());
        }

        let keys: Vec<&String> = result.extract_keys();
        self.write_header(&keys, false, true)?;

        for phase_result in &result.phases {
            let phase_data = PhaseRowData::new(
                &phase_result.name,
                phase_result.start_token.as_deref(),
                phase_result.end_token.as_deref(),
                phase_result.start_line,
                phase_result.end_line,
            );

            self.write_row_phase(cmd, phase_result, None, &phase_data)?;
        }

        self.finalize();
        Ok(())
    }

    fn phases_iterations(
        &mut self,
        cmd: &[String],
        _token_pattern: &str,
        results: &[PhaseMeasurementResult],
    ) -> Result<()> {
        if results.is_empty() {
            return Ok(());
        }

        let first_result = &results[0];

        if first_result.phases.is_empty() {
            return Ok(());
        }

        let mut keys: HashSet<&String> = HashSet::new();
        for phase_result in results {
            keys.extend(phase_result.extract_keys());
        }
        let keys_vec: Vec<&String> = keys.into_iter().collect();

        self.write_header(&keys_vec, true, true)?;

        for (idx, iteration_results) in results.iter().enumerate() {
            for phase_result in &iteration_results.phases {
                let phase_data = PhaseRowData::new(
                    &phase_result.name,
                    phase_result.start_token.as_deref(),
                    phase_result.end_token.as_deref(),
                    phase_result.start_line,
                    phase_result.end_line,
                );

                self.write_row_phase(cmd, phase_result, Some(idx), &phase_data)?;
            }
        }

        self.finalize();
        Ok(())
    }
}

impl ListSensorsDisplayer for CsvOutput {
    fn list_sensors(&mut self, sensors: &[Sensor]) -> Result<()> {
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
    }
}
