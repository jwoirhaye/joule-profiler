use std::{
    fs::File,
    io::{BufRead, BufReader, ErrorKind, Write},
    process::{self, Stdio},
};

use anyhow::{Context, Result, bail};
use log::{debug, info};
use regex::Regex;
use serde::Serialize;

use crate::{
    config::{Command, Config, Mode, PhasesConfig, ProfileConfig},
    core::{
        displayer::Displayer,
        metric::Metrics,
        orchestrator::SourceOrchestrator,
        phase::{PhaseInfo, PhaseToken},
        sensor::{Sensor, Sensors},
        source::{GetSensorsTrait, MetricReader, MetricSource, MetricSourceWorker},
    },
    error::JouleProfilerError,
    sources::rapl::Rapl,
    util::{command::run_command, file::create_file_with_user_permissions, time::get_timestamp},
};

#[derive(Debug, Serialize)]
pub struct Phase {
    pub start_token: PhaseToken,

    pub end_token: PhaseToken,

    pub timestamp: u128,

    pub duration_ms: u128,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_number: Option<usize>,

    pub metrics: Metrics,
}

impl Phase {
    pub fn new(
        metrics: Metrics,
        start_token: PhaseToken,
        end_token: PhaseToken,
        timestamp: u128,
        duration_ms: u128,
        line_number: Option<usize>,
    ) -> Self {
        Self {
            metrics,
            start_token,
            end_token,
            timestamp,
            duration_ms,
            line_number,
        }
    }
}

#[derive(Serialize)]
pub struct Iteration {
    pub index: usize,
    pub timestamp: u128,
    pub duration_ms: u128,
    pub exit_code: i32,
    pub measure_count: u64,
    pub measure_delta: u64,
    pub phases: Vec<Phase>,
}

impl Iteration {
    pub fn new(
        phases: Vec<Phase>,
        index: usize,
        timestamp: u128,
        duration_ms: u128,
        exit_code: i32,
        measure_count: u64,
        measure_delta: u64,
    ) -> Self {
        Self {
            phases,
            index,
            timestamp,
            duration_ms,
            exit_code,
            measure_count,
            measure_delta,
        }
    }
}

pub struct JouleProfiler {
    config: Config,
    orchestrator: SourceOrchestrator,
    displayer: Box<dyn Displayer>,
    sources: Vec<Box<dyn MetricSourceWorker>>,
}

impl TryFrom<Config> for JouleProfiler {
    type Error = anyhow::Error;

    fn try_from(config: Config) -> Result<Self> {
        let orchestrator = SourceOrchestrator::new();
        let displayer = (&config).try_into()?;
        let rapl = Rapl::try_from(&config)?;
        let sources = vec![rapl.into()];

        Ok(Self {
            orchestrator,
            config,
            displayer,
            sources,
        })
    }
}

impl JouleProfiler {
    /// Run Joule Profiler.
    pub async fn run(&mut self) -> Result<()> {
        match self.config.mode.clone() {
            Command::Profile(profile_config) => self.profile(profile_config).await,
            Command::ListSensors(_) => self.run_list_sensors(),
        }
    }

    pub fn add_source<T>(&mut self, reader: T)
    where
        T: MetricReader + GetSensorsTrait + Send + 'static,
        MetricSource<T>: Clone,
        T::Type: Send,
    {
        self.sources.push(reader.into());
    }

    async fn profile(&mut self, profile_config: ProfileConfig) -> Result<()> {
        match profile_config.mode.clone() {
            Mode::SimpleMode => self.run_simple(profile_config).await,
            Mode::PhaseMode(phases_config) => self.run_phases(profile_config, phases_config).await,
        }
    }

    fn run_list_sensors(&mut self) -> Result<()> {
        let sensors: Vec<Sensor> = self
            .sources
            .iter()
            .map(|source| source.list_sensors())
            .collect::<Result<Vec<Sensors>>>()?
            .into_iter()
            .flatten()
            .collect();

        self.displayer.list_sensors(&sensors)?;
        Ok(())
    }

    async fn run_simple(&mut self, config: ProfileConfig) -> Result<()> {
        info!("Running simple mode");

        self.orchestrator.start(self.sources.clone()).await;

        let mut command_results = Vec::with_capacity(config.iterations);

        debug!("Simple mode with {} iteration(s)", config.iterations);
        for _ in 0..config.iterations {
            let iteration = self.measure_simple(&config).await?;
            command_results.push(iteration);
        }

        let sources_results = self.orchestrator.retrieve().await?;

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
                            );
                            vec![phase]
                        } else {
                            Vec::new()
                        };

                    Iteration::new(
                        phases,
                        index,
                        begin_timestamp,
                        duration_ms,
                        exit_code,
                        iteration.measure_count,
                        iteration.measure_delta,
                    )
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

    async fn run_phases(
        &mut self,
        config: ProfileConfig,
        phases_config: PhasesConfig,
    ) -> Result<()> {
        self.orchestrator.start(self.sources.clone()).await;

        let mut command_results = Vec::with_capacity(config.iterations);

        for _ in 0..config.iterations {
            let iteration = self.measure_phases(&config, &phases_config).await?;
            command_results.push(iteration);
        }

        let sources_results = self.orchestrator.retrieve().await?;

        let results: Vec<Iteration> = command_results
            .into_iter()
            .zip(sources_results.iterations.into_iter().enumerate())
            .map(
                |(
                    (duration_ms, begin_timestamp, exit_code, detected_phases),
                    (index, iteration),
                )| {
                    let mut phases: Vec<_> = detected_phases
                        .windows(2)
                        .into_iter()
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
                                d1.duration_ms,
                                d1.line_number,
                            )
                        })
                        .collect();

                    if phases.is_empty()
                        && let Some(end_phase) = iteration.phases.into_iter().last().take()
                    {
                        phases.push(Phase::new(
                            end_phase.metrics,
                            PhaseToken::Start,
                            PhaseToken::End,
                            begin_timestamp,
                            duration_ms,
                            None,
                        ));
                    }

                    Iteration::new(
                        phases,
                        index,
                        begin_timestamp,
                        duration_ms,
                        exit_code,
                        iteration.measure_count,
                        iteration.measure_delta,
                    )
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
        Ok(())
    }

    async fn measure_simple(&self, config: &ProfileConfig) -> Result<(u128, u128, i32)> {
        self.orchestrator.new_iteration().await?;
        self.orchestrator.new_phase().await?;
        self.orchestrator.start_polling().await?;
        self.orchestrator.measure().await?;

        let begin_timestamp = get_timestamp();

        let (exit_code, _) = run_command(&config.cmd, config.output_file.as_ref())?;

        let end_timestamp = get_timestamp();

        self.orchestrator.measure().await?;
        self.orchestrator.stop_polling().await?;
        self.orchestrator.new_phase().await?;
        self.orchestrator.new_iteration().await?;

        let duration_ms = end_timestamp - begin_timestamp;

        Ok((duration_ms, begin_timestamp, exit_code))
    }

    async fn measure_phases(
        &self,
        config: &ProfileConfig,
        phases_config: &PhasesConfig,
    ) -> Result<(u128, u128, i32, Vec<PhaseInfo>)> {
        let regex = Regex::new(&phases_config.token_pattern).map_err(|e| {
            JouleProfilerError::InvalidPattern(format!("{}: {}", phases_config.token_pattern, e))
        })?;

        self.orchestrator.new_iteration().await?;
        self.orchestrator.new_phase().await?;
        self.orchestrator.start_polling().await?;
        self.orchestrator.measure().await?;

        let begin_timestamp = get_timestamp();
        let mut current_phase_token = PhaseToken::Start;
        let mut current_phase_timestamp = begin_timestamp;
        let mut current_phase_line_number = None;

        let mut command = process::Command::new(&config.cmd[0]);
        if config.cmd.len() > 1 {
            command.args(&config.cmd[1..]);
        }

        command.stdout(Stdio::piped());
        command.stderr(Stdio::inherit());

        let mut child = command.spawn().map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                JouleProfilerError::CommandNotFound(config.cmd[0].clone())
            } else {
                JouleProfilerError::CommandExecutionFailed(e.to_string())
            }
        })?;

        let stdout = child
            .stdout
            .take()
            .context("Failed to capture child stdout")?;

        let reader = BufReader::new(stdout);

        let mut output_file: Option<File> = if let Some(path) = &config.output_file {
            let file = create_file_with_user_permissions(path).map_err(|e| {
                JouleProfilerError::OutputFileCreationFailed(format!("{:?}: {}", path, e))
            })?;
            Some(file)
        } else {
            None
        };

        let mut detected_phases = Vec::new();

        for (line_number, line_res) in reader.lines().enumerate() {
            let line = match line_res {
                Ok(l) => l,
                Err(e) if e.kind() == ErrorKind::InvalidData => {
                    continue;
                }
                Err(e) => {
                    bail!(
                        "Failed to read line {} from command output: {}",
                        line_number + 1,
                        e
                    );
                }
            };

            if let Some(f) = output_file.as_mut() {
                writeln!(f, "{}", line)?;
            } else {
                println!("{}", line);
            }

            // Check if line matches the regex pattern
            if let Some(captures) = regex.captures(&line) {
                // Get the full match or the first capture group
                let token = if let Some(capture) = captures.get(1) {
                    capture.as_str().to_string()
                } else {
                    captures.get(0).unwrap().as_str().to_string()
                };

                let phase_timestamp = get_timestamp();
                let phase_duration = phase_timestamp - current_phase_timestamp;

                self.orchestrator.measure().await?;
                self.orchestrator.new_phase().await?;

                let phase_token =
                    std::mem::replace(&mut current_phase_token, PhaseToken::Token(token));
                let phase_info = PhaseInfo::new(
                    phase_token,
                    current_phase_timestamp,
                    phase_duration,
                    current_phase_line_number,
                );
                current_phase_timestamp = phase_timestamp;
                current_phase_line_number = Some(line_number);

                detected_phases.push(phase_info);
            }
        }

        let end_timestamp = get_timestamp();

        self.orchestrator.measure().await?;
        self.orchestrator.stop_polling().await?;
        self.orchestrator.new_phase().await?;
        self.orchestrator.new_iteration().await?;

        let phase_info = PhaseInfo::new(PhaseToken::End, end_timestamp, 0, None);
        detected_phases.push(phase_info);

        let status = child.wait().context("Failed to wait on child")?;
        let exit_code = status.code().unwrap_or(1);
        let duration_ms = end_timestamp - begin_timestamp;

        Ok((duration_ms, begin_timestamp, exit_code, detected_phases))
    }
}
