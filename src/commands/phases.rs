use std::{
    fs::File,
    io::{BufRead, BufReader, ErrorKind},
    process::{Command, Stdio},
};

use anyhow::{Context, Result, bail};
use regex::Regex;
use std::io::Write;
use tokio::time::Instant;

use crate::{
    config::profile::{PhasesConfig, ProfileConfig},
    core::{
        manager::SourceManager,
        measurement::{PhaseMeasurementResult, PhaseResult},
        phase::{PhaseInfo, PhaseToken},
    },
    error::JouleProfilerError,
    util::file::create_file_with_user_permissions,
};

pub async fn measure_phases(
    manager: &mut SourceManager,
    config: &ProfileConfig,
    phases_config: &PhasesConfig,
) -> Result<PhaseMeasurementResult> {
    let regex = Regex::new(&phases_config.token_pattern).map_err(|e| {
        JouleProfilerError::InvalidPattern(format!("{}: {}", phases_config.token_pattern, e))
    })?;

    let mut phases = Vec::new();

    manager.start().await?;

    let begin_instant = Instant::now();

    phases.push(PhaseInfo {
        token: PhaseToken::Start,
        timestamp: begin_instant,
        line_number: None,
    });

    manager.measure().await?;

    let mut command = Command::new(&config.cmd[0]);
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

    let mut out_file: Option<File> = if let Some(path) = &config.output_file {
        let file = create_file_with_user_permissions(path).map_err(|e| {
            JouleProfilerError::OutputFileCreationFailed(format!("{:?}: {}", path, e))
        })?;
        Some(file)
    } else {
        None
    };

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

        if let Some(f) = out_file.as_mut() {
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

            let phase_instant = Instant::now();

            manager.phase().await?;

            phases.push(PhaseInfo {
                token: PhaseToken::Token(token),
                timestamp: phase_instant,
                line_number: Some(line_number + 1),
            });
        }
    }

    let status = child.wait().context("Failed to wait on child")?;
    let end_instant = Instant::now();

    manager.measure().await?;

    let exit_code = status.code().unwrap_or(1);

    phases.push(PhaseInfo {
        token: PhaseToken::End,
        timestamp: end_instant,
        line_number: None,
    });

    let sources_result = manager.retrieve().await?;
    let mut phases_measurements = Vec::with_capacity(phases.len());

    for (i, phases) in phases.windows(2).enumerate() {
        let (begin_phase, end_phase) = (&phases[0], &phases[1]);
        let metrics = sources_result.measures[i].clone();
        let duration_ms = (end_phase.timestamp - begin_phase.timestamp).as_millis();

        let phase_mesurement = PhaseResult::new(
            &begin_phase.token,
            &end_phase.token,
            begin_phase.line_number,
            end_phase.line_number,
            metrics,
            duration_ms,
        );
        phases_measurements.push(phase_mesurement);
    }

    let duration_ms = (end_instant - begin_instant).as_millis();

    Ok(PhaseMeasurementResult {
        phases: phases_measurements,
        duration_ms,
        exit_code,
    })
}
