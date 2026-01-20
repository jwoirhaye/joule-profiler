use std::{
    fs::File,
    io::{BufRead, BufReader, ErrorKind, Write},
    process::{self, Stdio},
};

use log::{debug, info};
use regex::Regex;

pub mod error;

use crate::{
    config::{Command, Config, Mode, PhasesConfig, ProfileConfig},
    core::{
        displayer::Displayer,
        orchestrator::SourceOrchestrator,
        phase::{PhaseInfo, PhaseToken},
        profiler::{
            error::JouleProfilerError,
            types::{Iteration, Phase},
        },
        sensor::{Sensor, Sensors},
        source::{
            MetricSource, accumulator::MetricAccumulator, error::MetricSourceError,
            reader::MetricReader,
        },
    },
    sources::rapl::Rapl,
    util::{command::run_command, file::create_file_with_user_permissions, time::get_timestamp},
};

pub mod types;

type Result<T> = std::result::Result<T, JouleProfilerError>;

pub struct JouleProfiler {
    config: Config,
    orchestrator: SourceOrchestrator,
    displayer: Box<dyn Displayer>,
    sources: Vec<Box<dyn MetricSource>>,
}

impl TryFrom<Config> for JouleProfiler {
    type Error = JouleProfilerError;

    fn try_from(config: Config) -> Result<Self> {
        let orchestrator = SourceOrchestrator::new();

        let displayer = (&config).try_into()?;

        let rapl = Rapl::try_from(&config).map_err(MetricSourceError::from)?;
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
        T: MetricReader,
        MetricAccumulator<T>: Clone,
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
            .map(|source| source.list_sensors().map_err(MetricSourceError::into))
            .collect::<Result<Vec<Sensors>>>()?
            .into_iter()
            .flatten()
            .collect();

        self.displayer.list_sensors(&sensors)?;
        Ok(())
    }

    async fn run_simple(&mut self, config: ProfileConfig) -> Result<()> {
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
        let sources = std::mem::take(&mut self.sources);
        self.orchestrator.start(sources).await;

        let mut command_results = Vec::with_capacity(config.iterations);

        for _ in 0..config.iterations {
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
                )| {
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
                                d1.duration_ms,
                                d1.line_number,
                            )
                        })
                        .collect();

                    if phases.is_empty()
                        && let Some(end_phase) = iteration.phases.into_iter().last()
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

    async fn measure_simple(&mut self, config: &ProfileConfig) -> Result<(u128, u128, i32)> {
        self.orchestrator.start_polling().await?;
        self.orchestrator.measure().await?;

        let begin_timestamp = get_timestamp();

        let (exit_code, _) = run_command(&config.cmd, config.stdout_file.as_ref())?;

        let end_timestamp = get_timestamp();

        self.orchestrator.measure().await?;
        self.orchestrator.stop_polling().await?;
        self.orchestrator.new_phase().await?;
        self.orchestrator.new_iteration().await?;

        let duration_ms = end_timestamp - begin_timestamp;

        Ok((duration_ms, begin_timestamp, exit_code))
    }

    async fn measure_phases(
        &mut self,
        config: &ProfileConfig,
        phases_config: &PhasesConfig,
    ) -> Result<(u128, u128, i32, Vec<PhaseInfo>)> {
        let regex = Regex::new(&phases_config.token_pattern).map_err(|err| {
            JouleProfilerError::InvalidPattern(format!("{}: {}", phases_config.token_pattern, err))
        })?;

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

        let mut output_file: Option<File> = if let Some(path) = &config.stdout_file {
            let file = create_file_with_user_permissions(path).map_err(|err| {
                JouleProfilerError::OutputFileCreationFailed(format!("{:?}: {}", path, err))
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
                Err(e) => return Err(e.into()),
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
                    // Safe unwrap because if regex captures is some then it always has the full matching group
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

        let status = child.wait()?;
        let exit_code = status.code().unwrap_or(1);
        let duration_ms = end_timestamp - begin_timestamp;

        Ok((duration_ms, begin_timestamp, exit_code, detected_phases))
    }
}
