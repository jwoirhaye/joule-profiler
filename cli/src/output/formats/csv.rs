use std::fs::File;
use std::io::Write;

use joule_profiler_core::profiler::types::{Iteration, Phase};
use joule_profiler_core::sensor::Sensor;
use joule_profiler_core::util::file::{
    create_file_with_user_permissions, default_iterations_filename, get_absolute_path,
};

use crate::output::displayer::{Displayer, DisplayerError};

type Result<T> = std::result::Result<T, DisplayerError>;

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
    fn write_header(&mut self, with_iteration_id: bool) -> Result<()> {
        if with_iteration_id {
            write!(self.file, "iteration_id;")?;
        }

        write!(self.file, "phase_id;phase_name;phase_duration_ms;")?;
        write!(
            self.file,
            "metric_name;metric_value;metric_unit;metric_source;"
        )?;
        write!(
            self.file,
            "start_token;end_token;start_line;end_line;timestamp;"
        )?;
        write!(self.file, "command;exit_code;token_pattern")?;
        writeln!(self.file)?;

        Ok(())
    }

    /// Write a CSV row for a single phase.
    fn write_phase(
        &mut self,
        phase: &Phase,
        iteration: &Iteration,
        cmd: &str,
        token_pattern: &str,
        with_iteration_index: bool,
    ) -> Result<()> {
        for metric in &phase.metrics {
            if with_iteration_index {
                write!(self.file, "{};", iteration.index)?;
            }

            let start_line = phase.start_line.map(|l| l.to_string()).unwrap_or_default();
            let end_line = phase.end_line.map(|l| l.to_string()).unwrap_or_default();

            write!(
                self.file,
                "{};\"{}\";{};",
                phase.index,
                phase.get_name(),
                phase.duration_ms
            )?;
            write!(
                self.file,
                "{};{};{};{};",
                metric.name, metric.value, metric.unit, metric.source
            )?;
            write!(
                self.file,
                "{};{};{};{};{};",
                phase.start_token, phase.end_token, start_line, end_line, phase.timestamp
            )?;
            write!(
                self.file,
                "\"{}\";{};\"{}\"",
                cmd, iteration.exit_code, token_pattern
            )?;
            writeln!(self.file)?;
        }

        Ok(())
    }

    fn write_iteration(
        &mut self,
        iteration: &Iteration,
        cmd: &str,
        token_pattern: &str,
        with_iteration_index: bool,
    ) -> Result<()> {
        for phase in &iteration.phases {
            self.write_phase(phase, iteration, cmd, token_pattern, with_iteration_index)?;
        }
        Ok(())
    }

    /// Print a message indicating the CSV file has been written.
    fn finalize(&self) {
        println!("✔ CSV written to: {}", self.filename);
    }
}

impl Displayer for CsvOutput {
    fn phases_single(
        &mut self,
        cmd: &[String],
        token_pattern: &str,
        result: &Iteration,
    ) -> Result<()> {
        if result.phases.is_empty() {
            return Ok(());
        }

        let command = cmd.join(" ");

        self.write_header(false)?;
        self.write_iteration(result, command.as_str(), token_pattern, false)?;

        self.finalize();
        Ok(())
    }

    fn phases_iterations(
        &mut self,
        cmd: &[String],
        token_pattern: &str,
        iterations: &[Iteration],
    ) -> Result<()> {
        if iterations.is_empty() || iterations[0].phases.is_empty() {
            return Ok(());
        }

        let command = cmd.join(" ");

        self.write_header(true)?;
        for iteration in iterations {
            self.write_iteration(iteration, &command, token_pattern, true)?;
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
