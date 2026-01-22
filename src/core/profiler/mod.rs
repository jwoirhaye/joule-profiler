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

use std::{
    fs::File,
    io::{BufRead, BufReader, ErrorKind, Write},
    process::{self, Stdio},
};

use log::{debug, info, trace};
use regex::Regex;

pub mod error;

pub use error::JouleProfilerError;

use crate::{
    cli::Cli,
    config::{Command, Config, Mode, PhasesConfig, ProfileConfig},
    core::{
        aggregate::iteration::SensorIteration,
        displayer::Displayer,
        orchestrator::SourceOrchestrator,
        phase::{PhaseInfo, PhaseToken},
        profiler::types::{Iteration, Phase},
        sensor::{Sensor, Sensors},
        source::{MetricSource, error::MetricSourceError, reader::MetricReader},
    },
    sources::Rapl,
    util::{
        command::run_command, file::create_file_with_user_permissions, logging::init_logging,
        time::get_timestamp_millis,
    },
};

pub mod types;

/// Result type for profiler operations
type Result<T> = std::result::Result<T, JouleProfilerError>;

type MeasureSimpleReturnType = (u128, u128, i32);
type MeasurePhasesReturnType = (u128, u128, i32, Vec<PhaseInfo>);

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
pub struct JouleProfiler {
    config: Config,
    orchestrator: SourceOrchestrator,
    displayer: Box<dyn Displayer>,
    sources: Vec<Box<dyn MetricSource>>,
}

impl JouleProfiler {
    /// Creates a JouleProfiler from the CLI arguments.
    ///
    /// Uses clap to parse arguments.
    pub fn from_cli() -> Result<Self> {
        let cli = Cli::from_args()?;
        init_logging(cli.verbose);

        info!("Starting JouleProfiler");
        debug!("CLI parsed successfully");
        trace!("CLI args: {:?}", cli);

        let config = Config::from(cli);
        trace!("Profiler config: {:?}", config);

        JouleProfiler::try_from(config)
    }

    /// Run the profiler asynchronously.
    ///
    /// Executes the configured command and collects metrics according
    /// to the selected mode (simple or phase-based). Also supports
    /// listing sensors.
    pub async fn run(&mut self) -> Result<()> {
        info!("Profiler run started");

        match self.config.command.clone() {
            Command::Profile(profile_config) => {
                info!("Entering profiling mode");
                self.profile(profile_config).await
            }
            Command::ListSensors(_) => {
                info!("Entering sensor listing mode");
                self.run_list_sensors()
            }
        }
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

    /// Profile in the configured mode
    async fn profile(&mut self, profile_config: ProfileConfig) -> Result<()> {
        match profile_config.mode.clone() {
            Mode::SimpleMode => self.run_simple(profile_config).await,
            Mode::PhaseMode(phases_config) => self.run_phases(profile_config, phases_config).await,
        }
    }

    /// List the sensors of the provided sources.
    pub fn run_list_sensors(&mut self) -> Result<()> {
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
        self.displayer.list_sensors(&sensors)?;
        Ok(())
    }

    /// Run simple profiling mode.
    pub async fn run_simple(&mut self, config: ProfileConfig) -> Result<()> {
        info!("Running simple mode");

        let sources = std::mem::take(&mut self.sources);
        self.orchestrator.start(sources).await;

        let mut command_results = Vec::with_capacity(config.iterations);

        debug!("Simple mode with {} iteration(s)", config.iterations);
        for _ in 0..config.iterations {
            let iteration = self.measure_simple(&config).await?;
            command_results.push(iteration);
        }

        let (sources_results, sources) = self.orchestrator.retrieve().await?;
        self.sources = sources;

        let results: Vec<Iteration> = command_results
            .into_iter()
            .zip(sources_results.iterations.into_iter().enumerate())
            .map(
                |((duration_ms, begin_timestamp, exit_code), (index, iteration))| {
                    let phases =
                        if let Some(mut phase) = iteration.phases.into_iter().take(1).next() {
                            phase.metrics.sort_by_key(|metric| metric.name.clone());
                            let phase = Phase::new(
                                phase.metrics,
                                PhaseToken::Start,
                                PhaseToken::End,
                                begin_timestamp,
                                duration_ms,
                                None,
                                None,
                            );

                            vec![phase]
                        } else {
                            Vec::new()
                        };

                    Iteration::new(phases, index, begin_timestamp, duration_ms, exit_code)
                },
            )
            .collect();

        if config.iterations > 1 {
            self.displayer.simple_iterations(&config.cmd, &results)?;
        } else {
            self.displayer.simple_single(&config.cmd, &results[0])?;
        }
        Ok(())
    }

    /// Run phase-based profiling mode.
    pub async fn run_phases(
        &mut self,
        config: ProfileConfig,
        phases_config: PhasesConfig,
    ) -> Result<()> {
        info!("Running phase-based profiling");
        debug!("Iterations: {}", config.iterations);
        debug!("Phase regex: {}", phases_config.token_pattern);

        let sources = std::mem::take(&mut self.sources);
        trace!("Starting orchestrator with {} source(s)", sources.len());
        self.orchestrator.start(sources).await;

        let mut command_results = Vec::with_capacity(config.iterations);

        for i in 0..config.iterations {
            info!("Starting iteration {}", i);
            let iteration = self.measure_phases(&config, &phases_config).await?;
            command_results.push(iteration);
        }

        let (sources_results, sources) = self.orchestrator.retrieve().await?;
        self.sources = sources;

        let results: Vec<Iteration> = command_results
            .into_iter()
            .zip(sources_results.iterations.into_iter().enumerate())
            .map(
                |(
                    (duration_ms, begin_timestamp, exit_code, detected_phases),
                    (index, iteration),
                ): (MeasurePhasesReturnType, (usize, SensorIteration))| {
                    let mut phases: Vec<_> = detected_phases
                        .windows(2)
                        .zip(&iteration.phases)
                        .map(|(window, real_phase)| {
                            let (d1, d2) = (&window[0], &window[1]);
                            let mut phase_metrics = real_phase.metrics.clone();
                            phase_metrics.sort_by_key(|metric| metric.name.clone());
                            Phase::new(
                                phase_metrics,
                                d1.token.clone(),
                                d2.token.clone(),
                                d1.timestamp,
                                d2.timestamp - d1.timestamp,
                                d1.line_number,
                                d2.line_number,
                            )
                        })
                        .collect();

                    if phases.is_empty()
                        && let Some(end_phase) = iteration.phases.into_iter().last()
                    {
                        let phase = Phase::new(
                            end_phase.metrics,
                            PhaseToken::Start,
                            PhaseToken::End,
                            begin_timestamp,
                            duration_ms,
                            None,
                            None,
                        );
                        phases.push(phase);
                    }

                    Iteration::new(phases, index, begin_timestamp, duration_ms, exit_code)
                },
            )
            .collect();

        if config.iterations > 1 {
            self.displayer.phases_iterations(
                &config.cmd,
                &phases_config.token_pattern,
                &results,
            )?;
        } else {
            self.displayer
                .phases_single(&config.cmd, &phases_config.token_pattern, &results[0])?;
        }

        debug!("Collected {} sensor iteration(s)", results.len());

        Ok(())
    }

    /// Measure one iteration in simple mode
    async fn measure_simple(&mut self, config: &ProfileConfig) -> Result<MeasureSimpleReturnType> {
        self.orchestrator.start_polling().await?;
        self.orchestrator.measure().await?;

        let begin_timestamp = get_timestamp_millis();

        let (exit_code, _) = run_command(&config.cmd, config.stdout_file.as_ref())?;

        let end_timestamp = get_timestamp_millis();

        self.orchestrator.measure().await?;
        self.orchestrator.stop_polling().await?;
        self.orchestrator.new_phase().await?;
        self.orchestrator.new_iteration().await?;

        let duration_ms = end_timestamp - begin_timestamp;

        Ok((duration_ms, begin_timestamp, exit_code))
    }

    /// Measure one iteration in phases mode
    async fn measure_phases(
        &mut self,
        config: &ProfileConfig,
        phases_config: &PhasesConfig,
    ) -> Result<MeasurePhasesReturnType> {
        debug!("Compiling phase regex");

        let regex = Regex::new(&phases_config.token_pattern).map_err(|err| {
            JouleProfilerError::InvalidPattern(format!("{}: {}", phases_config.token_pattern, err))
        })?;

        let mut output_file: Option<File> = if let Some(path) = &config.stdout_file {
            let file = create_file_with_user_permissions(path).map_err(|err| {
                JouleProfilerError::OutputFileCreationFailed(format!("{:?}: {}", path, err))
            })?;

            Some(file)
        } else {
            None
        };

        self.orchestrator.start_polling().await?;
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
            if err.kind() == std::io::ErrorKind::NotFound {
                JouleProfilerError::CommandNotFound(config.cmd[0].clone())
            } else {
                JouleProfilerError::CommandExecutionFailed(err.to_string())
            }
        })?;

        let stdout = child
            .stdout
            .take()
            .ok_or(JouleProfilerError::StdOutCaptureFail)?;

        let reader = BufReader::new(stdout);
        let mut detected_phases = Vec::with_capacity(2);

        let start_phase = PhaseInfo::new(PhaseToken::Start, begin_timestamp, None);
        detected_phases.push(start_phase);

        self.detect_and_handle_phases_from_program_output(
            &mut detected_phases,
            reader,
            &regex,
            output_file.as_mut(),
        )
        .await?;

        let end_timestamp = get_timestamp_millis();
        trace!("End timestamp: {}", end_timestamp);

        self.orchestrator.measure().await?;
        self.orchestrator.stop_polling().await?;
        self.orchestrator.new_phase().await?;
        self.orchestrator.new_iteration().await?;

        let duration_ms = end_timestamp - begin_timestamp;

        let end_phase_info = PhaseInfo::new(PhaseToken::End, end_timestamp, None);
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

    async fn detect_and_handle_phases_from_program_output<R: BufRead>(
        &mut self,
        phases: &mut Vec<PhaseInfo>,
        reader: R,
        regex: &Regex,
        output_file: Option<&mut File>,
    ) -> Result<()> {
        for (line_number, line_res) in reader.lines().enumerate() {
            let line = match line_res {
                Ok(l) => l,
                Err(e) if e.kind() == ErrorKind::InvalidData => {
                    trace!("Skipping invalid UTF-8 output at line {}", line_number);
                    continue;
                }
                Err(e) => return Err(e.into()),
            };
            trace!("STDOUT[{}]: {}", line_number, line);

            if let Some(mut f) = output_file.as_deref() {
                writeln!(f, "{}", line)?;
            } else {
                println!("{}", line);
            }

            if let Some(token) = has_phase_token_in_line(regex, &line) {
                let phase_timestamp = get_timestamp_millis();

                debug!("Detected phase at line {}, token '{}'", line_number, token);

                self.orchestrator.measure().await?;
                self.orchestrator.new_phase().await?;

                let phase_info =
                    PhaseInfo::new(PhaseToken::Token(token.to_owned()), phase_timestamp, Some(line_number));

                phases.push(phase_info);
            }
        }

        Ok(())
    }
}

impl TryFrom<Config> for JouleProfiler {
    type Error = JouleProfilerError;

    /// Convert a configuration into a ready-to-run profiler.
    ///
    /// Automatically sets up the displayer and built-in RAPL source.
    fn try_from(config: Config) -> Result<Self> {
        info!("Building JouleProfiler from config");
        trace!("Profiler config: {:?}", config);

        let orchestrator = SourceOrchestrator::new();
        let displayer = (&config).try_into()?;

        debug!("Initializing RAPL source");
        let rapl = Rapl::try_from(&config).map_err(MetricSourceError::from)?;

        Ok(Self {
            orchestrator,
            config,
            displayer,
            sources: vec![rapl.into()],
        })
    }
}

pub fn has_phase_token_in_line<'a>(regex: &Regex, line: &'a str) -> Option<&'a str> {
        regex.find(line).map(|mat| mat.as_str())
}

#[cfg(test)]
mod tests {
    use std::io::{BufReader, Cursor};

    use regex::Regex;

    use crate::{
        JouleProfiler,
        config::{Command, Config, ListSensorsConfig},
        core::{orchestrator::SourceOrchestrator, phase::PhaseToken},
        output::{OutputFormat, TerminalOutput},
        util::time::get_timestamp_millis,
    };

    fn joule_profiler() -> JouleProfiler {
        let list_sensors_config = ListSensorsConfig {
            output_format: OutputFormat::Terminal,
        };
        let config = Config {
            command: Command::ListSensors(list_sensors_config),
            output_file: None,
            output_format: OutputFormat::Terminal,
            rapl_path: None,
        };
        JouleProfiler {
            config,
            orchestrator: SourceOrchestrator::new(),
            displayer: TerminalOutput.into(),
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

        profiler
            .detect_and_handle_phases_from_program_output(&mut phases, reader, &regex, None)
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

        profiler
            .detect_and_handle_phases_from_program_output(&mut phases, reader, &regex, None)
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

        profiler
            .detect_and_handle_phases_from_program_output(&mut phases, reader, &regex, None)
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

        profiler
            .detect_and_handle_phases_from_program_output(&mut phases, reader, &regex, None)
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

        profiler
            .detect_and_handle_phases_from_program_output(&mut phases, reader, &regex, None)
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

        profiler
            .detect_and_handle_phases_from_program_output(&mut phases, reader, &regex, None)
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
                Some(temp_file.as_file_mut()),
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

        profiler
            .detect_and_handle_phases_from_program_output(&mut phases, reader, &regex, None)
            .await
            .unwrap();

        assert_eq!(phases.len(), 1);
    }
}
