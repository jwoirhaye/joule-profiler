//! Profiler module
//!
//! This module provides the main `JouleProfiler` struct, which orchestrates
//! the execution of commands, collection of metrics from various sources,
//! and displaying results in different output formats.
//!
//! # Overview
//!
//! - [`JouleProfiler`] is the main entry point for running profiling sessions.
//! - Metrics sources implement [`MetricReader`] and [`MetricSource`].
//! - The profiler supports simple and phase-based profiling modes.
//! - Display is handled via the [`Displayer`] trait, which can output to
//!   terminal, JSON, CSV, or custom formats.
//!
//! # Usage
//!
//! ```ignore
//! use joule_profiler::{JouleProfiler, config::Config};
//!
//! // We assume you have a Config variable
//! let mut profiler = JouleProfiler::try_from(config).unwrap();
//! ```

use log::{debug, info, trace};
use regex::Regex;
use std::io::BufWriter;
use std::{
    fs::File,
    io::{BufRead, BufReader, ErrorKind, Write},
    process::{self, Stdio},
};

pub mod error;

use crate::aggregate::iteration::SensorIteration;
use crate::config::ProfileConfig;
use crate::orchestrator::SourceOrchestrator;
use crate::phase::{PhaseInfo, PhaseToken};
use crate::profiler::types::{Iteration, Iterations, MeasurePhasesReturnType, Phase, Result};
use crate::sensor::{Sensor, Sensors};
use crate::source::{MetricReader, MetricSource, MetricSourceError};
use crate::util::fs::create_file_with_user_permissions;
use crate::util::time::get_timestamp_millis;
pub use error::JouleProfilerError;

pub mod types;

/// Main profiler orchestrating command execution and metrics collection.
///
/// `JouleProfiler` handles the following responsibilities:
/// 1. Executes a configured command (or list sensors command).
/// 2. Collects metrics from multiple sources implementing [`MetricReader`].
/// 3. Aggregates results into `Iteration`s and `Phase`s.
/// 4. Displays results via the configured [`Displayer`].
///
/// The profiler supports both simple and phase-based modes and can run
/// multiple iterations.
///
/// # Fields
///
/// - `config` ([`Config`]): Configuration for profiling session, including
///   command, iterations, RAPL options, and output format.
/// - `orchestrator` (`SourceOrchestrator`): Manages polling and aggregation
///   from all metric sources.
/// - `displayer` (Box<dyn [`Displayer`]>): Handles displaying or exporting
///   metrics to terminal, JSON, CSV, etc.
/// - `sources` (Vec<Box<dyn [`MetricSource`]>): Registered metric sources
///   to collect data from (e.g., RAPL, custom sensors).
#[derive(Default)]
pub struct JouleProfiler {
    orchestrator: SourceOrchestrator,
    sources: Vec<Box<dyn MetricSource>>,
}

impl JouleProfiler {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a custom metric source to the profiler.
    ///
    /// All sources must implement [`MetricReader`].
    pub fn add_source<T>(&mut self, reader: T)
    where
        T: MetricReader,
    {
        debug!("Registering additional metric source: {}", T::get_name());
        trace!("MetricReader type: {}", std::any::type_name::<T>());
        self.sources.push(reader.into());
    }

    /// List the sensors of the provided sources.
    pub fn run_list_sensors(&mut self) -> Result<Sensors> {
        debug!("Listing sensors from {} source(s)", self.sources.len());

        let sensors: Vec<Sensor> = self
            .sources
            .iter()
            .enumerate()
            .map(|(i, source)| {
                trace!("Querying sensors from source {}", i);
                source.list_sensors().map_err(MetricSourceError::into)
            })
            .collect::<Result<Vec<Sensors>>>()?
            .into_iter()
            .flatten()
            .collect();

        info!("Discovered {} sensor(s)", sensors.len());
        Ok(sensors)
    }

    /// Run phase-based profiling mode.
    pub async fn run_phases(&mut self, config: &ProfileConfig) -> Result<Iterations> {
        info!("Running phase-based profiling");
        debug!("Iterations: {}", config.iterations);
        debug!("Phase regex: {}", config.token_pattern);

        let sources = std::mem::take(&mut self.sources);
        trace!("Starting orchestrator with {} source(s)", sources.len());
        self.orchestrator.run(sources).await;

        let mut command_results = Vec::with_capacity(config.iterations);

        for i in 0..config.iterations {
            info!("Starting iteration {}", i);
            let iteration = self.measure_phases(config).await?;
            command_results.push(iteration);
        }

        let (sources_results, sources) = self.orchestrator.finalize().await?;
        self.sources = sources;

        let results: Iterations = command_results
            .into_iter()
            .zip(sources_results.iterations.into_iter().enumerate())
            .map(
                |(
                    (duration_ms, begin_timestamp, exit_code, detected_phases),
                    (index, iteration),
                ): (MeasurePhasesReturnType, (usize, SensorIteration))| {
                    let mut phases: Vec<_> = detected_phases
                        .windows(2)
                        .enumerate()
                        .zip(&iteration.phases)
                        .map(|((index, window), real_phase)| {
                            let (d1, d2) = (&window[0], &window[1]);
                            let mut phase_metrics = real_phase.metrics.clone();
                            phase_metrics.sort_by_key(|metric| metric.name.clone());
                            Phase {
                                index,
                                metrics: phase_metrics,
                                start_token: d1.token.clone(),
                                end_token: d2.token.clone(),
                                timestamp: d1.timestamp,
                                duration_ms: d2.timestamp - d1.timestamp,
                                start_line: d1.line_number,
                                end_line: d2.line_number,
                            }
                        })
                        .collect();

                    if phases.is_empty()
                        && let Some(end_phase) = iteration.phases.into_iter().last()
                    {
                        let phase = Phase {
                            index: 0,
                            metrics: end_phase.metrics,
                            start_token: PhaseToken::Start,
                            end_token: PhaseToken::End,
                            timestamp: begin_timestamp,
                            duration_ms,
                            start_line: None,
                            end_line: None,
                        };
                        phases.push(phase);
                    }

                    Iteration {
                        phases,
                        index,
                        timestamp: begin_timestamp,
                        duration_ms,
                        exit_code,
                    }
                },
            )
            .collect();

        debug!("Collected {} sensor iteration(s)", results.len());

        Ok(results)
    }

    /// Measure one iteration in phases mode
    async fn measure_phases(&mut self, config: &ProfileConfig) -> Result<MeasurePhasesReturnType> {
        debug!("Compiling phase regex");

        let regex = Regex::new(&config.token_pattern).map_err(|err| {
            JouleProfilerError::InvalidPattern(format!("{}: {}", config.token_pattern, err))
        })?;

        let mut file_sink: Option<BufWriter<File>> = if let Some(path) = &config.stdout_file {
            let file = create_file_with_user_permissions(path).map_err(|err| {
                JouleProfilerError::OutputFileCreationFailed(format!("{:?}: {}", path, err))
            })?;

            Some(BufWriter::new(file))
        } else {
            None
        };

        let mut stdout_sink = BufWriter::new(std::io::stdout().lock());

        let sink: &mut dyn Write = if let Some(f) = file_sink.as_mut() {
            f
        } else {
            &mut stdout_sink
        };

        self.orchestrator.measure().await?;

        let mut command = process::Command::new(&config.cmd[0]);
        let begin_timestamp = get_timestamp_millis();
        if config.cmd.len() > 1 {
            command.args(&config.cmd[1..]);
        }
        trace!("Begin timestamp: {}", begin_timestamp);

        command.stdout(Stdio::piped());
        command.stderr(Stdio::inherit());

        debug!("Spawning command: {:?}", config.cmd);

        let mut child = command.spawn().map_err(|err| {
            if err.kind() == ErrorKind::NotFound {
                JouleProfilerError::CommandNotFound(config.cmd[0].clone())
            } else {
                JouleProfilerError::CommandExecutionFailed(err.to_string())
            }
        })?;

        let child_stdout = child
            .stdout
            .take()
            .ok_or(JouleProfilerError::StdOutCaptureFail)?;

        let reader = BufReader::new(child_stdout);

        let mut detected_phases = Vec::with_capacity(2);
        let start_phase_info = PhaseInfo {
            token: PhaseToken::Start,
            timestamp: begin_timestamp,
            line_number: None,
        };
        detected_phases.push(start_phase_info);

        self.detect_and_handle_phases_from_program_output(
            &mut detected_phases,
            reader,
            &regex,
            sink,
        )
        .await?;

        sink.flush()?;

        let end_timestamp = get_timestamp_millis();
        trace!("End timestamp: {}", end_timestamp);

        self.orchestrator.measure().await?;
        self.orchestrator.new_phase().await?;
        self.orchestrator.new_iteration().await?;

        let duration_ms = end_timestamp - begin_timestamp;

        let end_phase_info = PhaseInfo {
            token: PhaseToken::End,
            timestamp: end_timestamp,
            line_number: None,
        };
        detected_phases.push(end_phase_info);

        let status = child.wait()?;
        let exit_code = status.code().unwrap_or(1);

        info!(
            "Command finished: duration={} µs exit_code={}",
            end_timestamp - begin_timestamp,
            exit_code
        );

        Ok((duration_ms, begin_timestamp, exit_code, detected_phases))
    }

    async fn detect_and_handle_phases_from_program_output<R, W>(
        &mut self,
        phases: &mut Vec<PhaseInfo>,
        mut reader: R,
        regex: &Regex,
        sink: &mut W,
    ) -> Result<()>
    where
        R: BufRead,
        W: Write + ?Sized,
    {
        let mut line = String::new();
        let mut line_number: usize = 0;

        loop {
            line.clear();

            let n = match reader.read_line(&mut line) {
                Ok(n) => n,
                Err(e) if e.kind() == ErrorKind::InvalidData => {
                    trace!("Skipping invalid UTF-8 output at line {}", line_number);
                    line_number += 1;
                    continue;
                }
                Err(e) => return Err(e.into()),
            };

            if n == 0 {
                break;
            }

            if line.ends_with('\n') {
                line.pop();
                if line.ends_with('\r') {
                    line.pop();
                }
            }

            trace!("STDOUT[{}]: {}", line_number, line);

            writeln!(sink, "{}", line)?;

            if let Some(token) = phase_token_in_line(regex, &line) {
                let phase_timestamp = get_timestamp_millis();

                debug!("Detected phase at line {}, token '{}'", line_number, token);

                self.orchestrator.measure().await?;
                self.orchestrator.new_phase().await?;

                let phase_info = PhaseInfo {
                    token: PhaseToken::Token(token.to_owned()),
                    timestamp: phase_timestamp,
                    line_number: Some(line_number),
                };

                phases.push(phase_info);
            }

            line_number += 1;
        }

        Ok(())
    }
}

pub fn phase_token_in_line<'a>(regex: &Regex, line: &'a str) -> Option<&'a str> {
    regex.find(line).map(|mat| mat.as_str())
}

#[cfg(test)]
mod tests {
    use std::io::{BufReader, Cursor};

    use regex::Regex;

    use crate::orchestrator::SourceOrchestrator;
    use crate::phase::PhaseToken;
    use crate::profiler::phase_token_in_line;
    use crate::{JouleProfiler, util::time::get_timestamp_millis};

    fn joule_profiler() -> JouleProfiler {
        JouleProfiler {
            orchestrator: SourceOrchestrator::default(),
            sources: Vec::new(),
        }
    }

    #[tokio::test]
    async fn detect_multiple_phases() {
        let mut profiler = joule_profiler();
        let regex = Regex::new("__[A-Z0-9_]+__").unwrap();
        let cursor = Cursor::new("__PHASE1__\n__PHASE2__\n__PHASE3__");
        let reader = BufReader::new(cursor);
        let mut phases = Vec::new();
        let mut sink: Vec<u8> = Vec::new();

        profiler
            .detect_and_handle_phases_from_program_output(&mut phases, reader, &regex, &mut sink)
            .await
            .unwrap();

        assert_eq!(3, phases.len());
        assert_eq!(PhaseToken::Token("__PHASE1__".to_string()), phases[0].token);
        assert_eq!(PhaseToken::Token("__PHASE2__".to_string()), phases[1].token);
        assert_eq!(PhaseToken::Token("__PHASE3__".to_string()), phases[2].token);
    }

    #[tokio::test]
    async fn detect_no_phases() {
        let mut profiler = joule_profiler();
        let regex = Regex::new("__[A-Z0-9_]+__").unwrap();
        let cursor = Cursor::new("hello\nworld\nno phases here");
        let reader = BufReader::new(cursor);
        let mut phases = Vec::new();
        let mut sink: Vec<u8> = Vec::new();
        profiler
            .detect_and_handle_phases_from_program_output(&mut phases, reader, &regex, &mut sink)
            .await
            .unwrap();

        assert!(phases.is_empty());
    }

    #[tokio::test]
    async fn detect_empty_output() {
        let mut profiler = joule_profiler();
        let regex = Regex::new("__PHASE__").unwrap();
        let cursor = Cursor::new("");
        let reader = BufReader::new(cursor);
        let mut phases = Vec::new();
        let mut sink: Vec<u8> = Vec::new();

        profiler
            .detect_and_handle_phases_from_program_output(&mut phases, reader, &regex, &mut sink)
            .await
            .unwrap();

        assert_eq!(phases.len(), 0);
    }

    #[tokio::test]
    async fn detect_phase_in_middle_of_line() {
        let mut profiler = joule_profiler();
        let regex = Regex::new("__PHASE[0-9]+__").unwrap();
        let cursor = Cursor::new("start __PHASE1__ end");
        let reader = BufReader::new(cursor);
        let mut phases = Vec::new();
        let mut sink: Vec<u8> = Vec::new();

        profiler
            .detect_and_handle_phases_from_program_output(&mut phases, reader, &regex, &mut sink)
            .await
            .unwrap();

        assert_eq!(phases.len(), 1);
        assert_eq!(phases[0].token, PhaseToken::Token("__PHASE1__".to_string()));
        assert_eq!(phases[0].line_number, Some(0));
    }

    #[tokio::test]
    async fn detect_correct_line_numbers() {
        let mut profiler = joule_profiler();
        let regex = Regex::new("__PHASE[0-9]+__").unwrap();
        let cursor = Cursor::new("a\nb\n__PHASE1__\nc\n__PHASE2__");
        let reader = BufReader::new(cursor);
        let mut phases = Vec::new();
        let mut sink: Vec<u8> = Vec::new();

        profiler
            .detect_and_handle_phases_from_program_output(&mut phases, reader, &regex, &mut sink)
            .await
            .unwrap();

        assert_eq!(phases.len(), 2);
        assert_eq!(phases[0].line_number, Some(2));
        assert_eq!(phases[1].line_number, Some(4));
    }

    #[tokio::test]
    async fn phase_durations_are_monotonic() {
        let mut profiler = joule_profiler();
        let regex = Regex::new("__PHASE[0-9]+__").unwrap();
        let cursor = Cursor::new("__PHASE1__\n__PHASE2__\n__PHASE3__");
        let reader = BufReader::new(cursor);
        let begin_timestamp = get_timestamp_millis();
        let mut phases = Vec::new();
        let mut sink: Vec<u8> = Vec::new();

        profiler
            .detect_and_handle_phases_from_program_output(&mut phases, reader, &regex, &mut sink)
            .await
            .unwrap();

        let mut last_phase_timestamp = begin_timestamp;

        for phase in &phases {
            assert!(phase.timestamp >= last_phase_timestamp);
            last_phase_timestamp = phase.timestamp;
        }
    }

    #[tokio::test]
    async fn writes_stdout_to_file() {
        use std::fs;
        use tempfile::NamedTempFile;

        let mut profiler = joule_profiler();
        let regex = Regex::new("__PHASE__").unwrap();
        let cursor = Cursor::new("hello\n__PHASE__\nworld");
        let reader = BufReader::new(cursor);

        let mut temp_file = NamedTempFile::new().unwrap();
        let mut phases = Vec::new();

        profiler
            .detect_and_handle_phases_from_program_output(
                &mut phases,
                reader,
                &regex,
                temp_file.as_file_mut(),
            )
            .await
            .unwrap();

        let content = fs::read_to_string(temp_file.path()).unwrap();
        assert!(content.contains("hello"));
        assert!(content.contains("__PHASE__"));
        assert!(content.contains("world"));
    }

    #[tokio::test]
    async fn skips_invalid_utf8_lines() {
        let mut profiler = joule_profiler();
        let regex = Regex::new("__PHASE__").unwrap();

        let bytes = vec![
            0xff, 0xfe, b'\n', b'_', b'_', b'P', b'H', b'A', b'S', b'E', b'_', b'_',
        ];
        let cursor = Cursor::new(bytes);
        let reader = BufReader::new(cursor);
        let mut phases = Vec::new();
        let mut sink: Vec<u8> = Vec::new();

        profiler
            .detect_and_handle_phases_from_program_output(&mut phases, reader, &regex, &mut sink)
            .await
            .unwrap();

        assert_eq!(phases.len(), 1);
    }

    #[test]
    fn phase_token_in_line_returns_none_when_no_match() {
        let regex = Regex::new("X").unwrap();
        assert_eq!(phase_token_in_line(&regex, "abc"), None);
    }

    #[test]
    fn phase_token_in_line_returns_some_when_match_exists() {
        let regex = Regex::new("X").unwrap();
        assert_eq!(phase_token_in_line(&regex, "aXc"), Some("X"));
    }

    #[test]
    fn phase_token_in_line_returns_first_match_only() {
        let regex = Regex::new("X").unwrap();
        assert_eq!(phase_token_in_line(&regex, "XX"), Some("X"));
    }

    #[test]
    fn phase_token_in_line_does_not_trim_or_modify_input() {
        let regex = Regex::new("X").unwrap();
        assert_eq!(phase_token_in_line(&regex, "  X  "), Some("X"));
    }

    #[test]
    fn phase_token_in_line_returns_slice_from_input() {
        let regex = Regex::new("X").unwrap();
        let line = String::from("aXc");

        let token = phase_token_in_line(&regex, &line).unwrap();

        let line_ptr = line.as_ptr() as usize;
        let tok_ptr = token.as_ptr() as usize;
        assert!(tok_ptr >= line_ptr && tok_ptr < line_ptr + line.len());
    }
}
