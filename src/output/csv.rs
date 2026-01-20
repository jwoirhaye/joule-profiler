use std::fs::File;
use std::io::Write;

use crate::core::displayer::{Displayer, Result, default_iterations_filename};
use crate::core::profiler::types::{Iteration, Phase};
use crate::core::sensor::Sensor;
use crate::util::file::{create_file_with_user_permissions, get_absolute_path};

pub struct CsvOutput {
    file: File,
    filename: String,
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

        for phase in &result.phases {
            self.write_row_phase(cmd, phase, None)?;
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

        for (idx, iteration_results) in iterations.iter().enumerate() {
            for phase in &iteration_results.phases {
                self.write_row_phase(cmd, phase, Some(idx))?;
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

impl CsvOutput {
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
        result: &Iteration,
        iteration: Option<usize>,
    ) -> Result<()> {
        write!(self.file, "'{}';", command.join(" "))?;

        let phase = &result.phases[0];

        if let Some(idx) = iteration {
            write!(self.file, "{};", idx)?;
        }

        for metric in &phase.metrics {
            write!(self.file, "{};", metric.value)?;
        }

        writeln!(
            self.file,
            "{};{};{};{};",
            result.duration_ms, result.measure_count, result.measure_delta, result.exit_code
        )?;

        Ok(())
    }

    fn write_row_phase(
        &mut self,
        command: &[String],
        phase: &Phase,
        iteration: Option<usize>,
    ) -> Result<()> {
        write!(self.file, "'{}';", command.join(" "))?;

        if let Some(idx) = iteration {
            write!(self.file, "{};", idx)?;
        }

        write!(self.file, "{};", phase.start_token)?;
        write!(self.file, "{};", phase.end_token)?;
        write!(
            self.file,
            "{};",
            phase.line_number.map(|l| l.to_string()).unwrap_or_default()
        )?;

        for metric in &phase.metrics {
            write!(self.file, "{};", metric.value)?;
        }

        writeln!(self.file, "{};", phase.duration_ms)?;

        Ok(())
    }

    fn finalize(&self) {
        println!("✔ CSV written to: {}", self.filename);
    }
}
