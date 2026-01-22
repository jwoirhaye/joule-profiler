use std::fs::File;
use std::io::Write;

use crate::core::displayer::{Displayer, Result, default_iterations_filename};
use crate::core::profiler::types::{Iteration, Phase};
use crate::core::sensor::Sensor;
use crate::util::file::{create_file_with_user_permissions, get_absolute_path};

/// CSV output writer to a file.
pub struct CsvOutput {
    /// File handle for writing CSV data.
    file: File,

    /// Path to the output CSV file.
    filename: String,
}

impl CsvOutput {
    /// Create a CSV output writer to a file, optionally specifying the file path.
    pub fn try_new(output_file: Option<String>) -> Result<Self> {
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

    /// Write CSV header row.
    fn write_header(
        &mut self,
        keys: &[&String],
        include_iteration: bool,
        include_phase: bool,
    ) -> Result<()> {
        if include_iteration {
            write!(self.file, "iteration;")?;
        }
        write!(self.file, "command;")?;

        if include_phase {
            write!(self.file, "start_token;end_token;start_line;end_line;")?;
        }

        for key in keys {
            write!(self.file, "{};", key)?;
        }

        writeln!(self.file, "duration_ms;exit_code")?;
        Ok(())
    }

    /// Write a CSV row for a simple iteration.
    fn write_row(
        &mut self,
        command: &[String],
        result: &Iteration,
        iteration: Option<usize>,
    ) -> Result<()> {
        if let Some(idx) = iteration {
            write!(self.file, "{};", idx)?;
        }
        write!(self.file, "'{}';", command.join(" "))?;

        let phase = &result.phases[0];
        for metric in &phase.metrics {
            write!(self.file, "{};", metric.value)?;
        }

        writeln!(self.file, "{};{}", result.duration_ms, result.exit_code)?;
        Ok(())
    }

    /// Write a CSV row for a single phase.
    fn write_phase_row(&mut self, phase: &Phase, iteration_index: Option<usize>) -> Result<()> {
        if let Some(i) = iteration_index {
            write!(self.file, "{};", i)?;
        }

        // command
        write!(self.file, ";")?;

        write!(
            self.file,
            "{};{};{};{};",
            phase.start_token,
            phase.end_token,
            phase.start_line.map(|l| l.to_string()).unwrap_or_default(),
            phase.end_line.map(|l| l.to_string()).unwrap_or_default(),
        )?;

        for metric in &phase.metrics {
            write!(self.file, "{};", metric.value)?;
        }

        write!(self.file, "{};", phase.duration_ms)?;

        // iteration collumns
        writeln!(self.file, ";;")?;

        Ok(())
    }

    fn write_iteration_row(
        &mut self,
        command: &[String],
        iteration: &Iteration,
        index: Option<usize>,
    ) -> Result<()> {
        // iteration index
        if let Some(i) = index {
            write!(self.file, "{};", i)?;
        }
        write!(self.file, "'{}';", command.join(" "))?;

        // phase columns
        write!(self.file, ";;;;")?;

        let phase = &iteration.phases[0];
        for _ in &phase.metrics {
            write!(self.file, ";")?;
        }

        write!(
            self.file,
            "{};{}",
            iteration.duration_ms, iteration.exit_code
        )?;

        Ok(())
    }

    /// Print a message indicating the CSV file has been written.
    fn finalize(&self) {
        println!("✔ CSV written to: {}", self.filename);
    }
}

impl Displayer for CsvOutput {
    fn simple_single(&mut self, cmd: &[String], result: &Iteration) -> Result<()> {
        let keys: Vec<&String> = result.phases[0]
            .metrics
            .iter()
            .map(|metric| &metric.name)
            .collect();

        self.write_header(&keys, false, false)?;
        self.write_row(cmd, result, None)?;

        self.finalize();
        Ok(())
    }

    fn simple_iterations(&mut self, cmd: &[String], results: &[Iteration]) -> Result<()> {
        if results.is_empty() {
            return Ok(());
        }

        let first = &results[0];
        let keys: Vec<&String> = first.phases[0]
            .metrics
            .iter()
            .map(|metric| &metric.name)
            .collect();

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
        result: &Iteration,
    ) -> Result<()> {
        if result.phases.is_empty() {
            return Ok(());
        }

        let keys: Vec<&String> = result.phases[0]
            .metrics
            .iter()
            .map(|metric| &metric.name)
            .collect();

        self.write_header(&keys, false, true)?;
        self.write_iteration_row(cmd, result, None)?;
        writeln!(self.file)?;

        for phase in &result.phases {
            self.write_phase_row(phase, None)?;
        }

        self.finalize();
        Ok(())
    }

    fn phases_iterations(
        &mut self,
        cmd: &[String],
        _token_pattern: &str,
        iterations: &[Iteration],
    ) -> Result<()> {
        if iterations.is_empty() {
            return Ok(());
        }

        let first_result = &iterations[0];

        if first_result.phases.is_empty() {
            return Ok(());
        }

        let keys: Vec<&String> = iterations[0].phases[0]
            .metrics
            .iter()
            .map(|metric| &metric.name)
            .collect();

        self.write_header(&keys, true, true)?;

        for (i, iteration) in iterations.iter().enumerate() {
            self.write_iteration_row(cmd, iteration, Some(i))?;
            writeln!(self.file)?;
            for phase in &iteration.phases {
                self.write_phase_row(phase, Some(i))?;
            }
        }

        self.finalize();
        Ok(())
    }

    fn list_sensors(&mut self, sensors: &[Sensor]) -> Result<()> {
        writeln!(self.file, "sensor;unit;source")?;

        for sensor in sensors {
            writeln!(
                self.file,
                "{};{};{}",
                sensor.name, sensor.unit, sensor.source
            )?;
        }

        self.finalize();
        Ok(())
    }
}
