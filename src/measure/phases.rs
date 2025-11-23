use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

use anyhow::{Context, Result};
use log::{debug, error, info, trace, warn};
use regex::Regex;

use crate::config::Config;
use crate::errors::JouleProfilerError;
use crate::measure::common::{
    PhaseMeasurement, PhasesResult, build_max_map, compute_measurement_from_snapshots,
};
use crate::rapl::{EnergySnapshot, RaplDomain, read_snapshot};

/// Detected token with timestamp
#[derive(Debug, Clone)]
struct DetectedToken {
    token: String,
    snapshot: EnergySnapshot,
    line_number: usize,
}

/// Measure one run in phases mode with regex pattern.
pub fn measure_phases_once(config: &Config, domains: &[RaplDomain]) -> Result<PhasesResult> {
    info!("Starting single phase measurement with regex pattern");

    if config.cmd.is_empty() {
        error!("No command specified for measurement");
        return Err(JouleProfilerError::NoCommand.into());
    }

    let token_pattern = config.token_pattern.as_ref().ok_or_else(|| {
        error!("token_pattern not configured for phases mode");
        JouleProfilerError::TokenNotFound("token_pattern not configured".to_string())
    })?;

    info!(
        "Running in phase mode with token pattern: '{}'",
        token_pattern
    );

    let regex = Regex::new(token_pattern).map_err(|e| {
        error!("Invalid regex pattern '{}': {}", token_pattern, e);
        JouleProfilerError::InvalidPattern(format!("{}: {}", token_pattern, e))
    })?;

    let filtered: Vec<&RaplDomain> = domains
        .iter()
        .filter(|d| config.sockets.contains(&d.socket))
        .collect();

    if filtered.is_empty() {
        error!(
            "No RAPL domains found for requested sockets {:?}",
            config.sockets
        );
        return Err(JouleProfilerError::NoDomains.into());
    }

    debug!(
        "Filtered {} domain(s) for sockets {:?}",
        filtered.len(),
        config.sockets
    );

    let max_map = build_max_map(&filtered);
    trace!("Built max_energy map with {} entries", max_map.len());

    let start_snapshot = read_snapshot(&filtered)?;
    info!(
        "Initial snapshot taken at {} µs",
        start_snapshot.timestamp_us
    );

    let mut command = Command::new(&config.cmd[0]);
    if config.cmd.len() > 1 {
        command.args(&config.cmd[1..]);
    }

    command.stdout(Stdio::piped());
    command.stderr(Stdio::inherit());

    debug!("Spawning command: {:?}", config.cmd);

    let mut child = command.spawn().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            error!("Command not found: {}", config.cmd[0]);
            JouleProfilerError::CommandNotFound(config.cmd[0].clone())
        } else {
            error!("Failed to spawn command: {}", e);
            JouleProfilerError::CommandExecutionFailed(e.to_string())
        }
    })?;

    info!("Command spawned successfully (PID: {:?})", child.id());

    let stdout = child
        .stdout
        .take()
        .context("Failed to capture child stdout")?;
    let reader = BufReader::new(stdout);

    let mut out_file: Option<File> = if let Some(path) = config.output_file.as_deref() {
        debug!("Creating output file: {:?}", path);
        Some(File::create(path).map_err(|e| {
            error!("Failed to create output file {:?}: {}", path, e);
            JouleProfilerError::OutputFileCreationFailed(format!("{:?}: {}", path, e))
        })?)
    } else {
        debug!("No output file specified, using stdout");
        None
    };

    let mut detected_tokens = Vec::<DetectedToken>::new();
    let mut line_count = 0;

    trace!(
        "Starting to monitor command output for tokens matching pattern '{}'",
        token_pattern
    );

    for line_res in reader.lines() {
        let line = line_res?;
        line_count += 1;

        // Write to output file or stdout
        if let Some(f) = out_file.as_mut() {
            writeln!(f, "{}", line).map_err(|e| {
                error!("Failed to write to output file: {}", e);
                JouleProfilerError::OutputWriteFailed(e.to_string())
            })?;
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

            info!("✓ Detected token '{}' at line {}", token, line_count);

            let snapshot = read_snapshot(&filtered)?;
            debug!(
                "Token '{}' (line {}) snapshot taken at {} µs",
                token, line_count, snapshot.timestamp_us
            );

            detected_tokens.push(DetectedToken {
                token: token.clone(),
                snapshot,
                line_number: line_count,
            });
        }
    }

    debug!("Processed {} lines of command output", line_count);
    info!("Found {} matching token(s)", detected_tokens.len());

    let status = child.wait().context("Failed to wait on child")?;
    let exit_code = status.code().unwrap_or_else(|| {
        warn!("Command terminated by signal, using exit code 1");
        1
    });

    if exit_code == 0 {
        info!("Command completed successfully (exit code: 0)");
    } else {
        warn!("Command failed with exit code: {}", exit_code);
    }

    let end_snapshot = read_snapshot(&filtered)?;
    info!("Final snapshot taken at {} µs", end_snapshot.timestamp_us);

    // Build phases from detected tokens
    let mut phases = Vec::<PhaseMeasurement>::new();

    let duration_between_ms = |a: &EnergySnapshot, b: &EnergySnapshot| -> u128 {
        if b.timestamp_us >= a.timestamp_us {
            (b.timestamp_us - a.timestamp_us) / 1000
        } else {
            warn!(
                "Negative time duration detected: {} -> {}",
                a.timestamp_us, b.timestamp_us
            );
            0
        }
    };

    // 1. Global phase (START -> END)
    debug!("Computing global phase (START -> END)");
    let global_duration_ms = duration_between_ms(&start_snapshot, &end_snapshot);
    let global = compute_measurement_from_snapshots(
        &filtered,
        &max_map,
        &start_snapshot,
        &end_snapshot,
        global_duration_ms,
        exit_code,
    )?;
    info!("Global phase: {} ms", global_duration_ms);
    phases.push(PhaseMeasurement {
        name: "global (START -> END)".to_string(),
        start_token: Some("START".to_string()),
        end_token: Some("END".to_string()),
        start_line: None,
        end_line: None,
        result: global,
    });

    if detected_tokens.is_empty() {
        warn!(
            "⚠ No tokens matching pattern '{}' were detected in output",
            token_pattern
        );
    } else {
        // 2. Phase from START to first token
        let first_token = &detected_tokens[0];
        debug!(
            "Computing phase START -> '{}' (line {})",
            first_token.token, first_token.line_number
        );
        let duration_ms = duration_between_ms(&start_snapshot, &first_token.snapshot);
        let measurement = compute_measurement_from_snapshots(
            &filtered,
            &max_map,
            &start_snapshot,
            &first_token.snapshot,
            duration_ms,
            exit_code,
        )?;
        info!("Phase START -> '{}': {} ms", first_token.token, duration_ms);
        phases.push(PhaseMeasurement {
            name: format!("START -> {}", first_token.token),
            start_token: Some("START".to_string()),
            end_token: Some(first_token.token.clone()),
            start_line: None,
            end_line: Some(first_token.line_number),
            result: measurement,
        });

        // 3. Phases between consecutive tokens
        for i in 0..detected_tokens.len() - 1 {
            let token_a = &detected_tokens[i];
            let token_b = &detected_tokens[i + 1];

            debug!(
                "Computing phase '{}' (line {}) -> '{}' (line {})",
                token_a.token, token_a.line_number, token_b.token, token_b.line_number
            );
            let duration_ms = duration_between_ms(&token_a.snapshot, &token_b.snapshot);
            let measurement = compute_measurement_from_snapshots(
                &filtered,
                &max_map,
                &token_a.snapshot,
                &token_b.snapshot,
                duration_ms,
                exit_code,
            )?;
            info!(
                "Phase '{}' -> '{}': {} ms",
                token_a.token, token_b.token, duration_ms
            );
            phases.push(PhaseMeasurement {
                name: format!("{} -> {}", token_a.token, token_b.token),
                start_token: Some(token_a.token.clone()),
                end_token: Some(token_b.token.clone()),
                start_line: Some(token_a.line_number),
                end_line: Some(token_b.line_number),
                result: measurement,
            });
        }

        // 4. Phase from last token to END
        let last_token = &detected_tokens[detected_tokens.len() - 1];
        debug!(
            "Computing phase '{}' (line {}) -> END",
            last_token.token, last_token.line_number
        );
        let duration_ms = duration_between_ms(&last_token.snapshot, &end_snapshot);
        let measurement = compute_measurement_from_snapshots(
            &filtered,
            &max_map,
            &last_token.snapshot,
            &end_snapshot,
            duration_ms,
            exit_code,
        )?;
        info!("Phase '{}' -> END: {} ms", last_token.token, duration_ms);
        phases.push(PhaseMeasurement {
            name: format!("{} -> END", last_token.token),
            start_token: Some(last_token.token.clone()),
            end_token: Some("END".to_string()),
            start_line: Some(last_token.line_number),
            end_line: None,
            result: measurement,
        });
    }

    info!(
        "Phase measurement completed: {} phase(s) computed",
        phases.len()
    );

    Ok(PhasesResult { phases })
}

/// Run phases measurement N times.
pub fn measure_phases_iterations(
    config: &Config,
    domains: &[RaplDomain],
    iterations: usize,
) -> Result<Vec<(usize, PhasesResult)>> {
    if iterations == 0 {
        return Err(JouleProfilerError::InvalidIterations(0).into());
    }

    info!("Starting {} phase measurement iteration(s)", iterations);

    let mut all = Vec::with_capacity(iterations);

    for i in 0..iterations {
        info!("═══ Phase iteration {}/{} ═══", i + 1, iterations);

        match measure_phases_once(config, domains) {
            Ok(res) => {
                debug!("Iteration {} completed successfully", i + 1);
                all.push((i, res));
            }
            Err(e) => {
                error!("Iteration {} failed: {}", i + 1, e);
                return Err(e);
            }
        }
    }

    info!("All {} iteration(s) completed successfully", iterations);

    Ok(all)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn create_mock_domain(name: &str, socket: u32) -> RaplDomain {
        RaplDomain {
            path: PathBuf::from(format!(
                "/sys/class/powercap/intel-rapl:0/{}/energy_uj",
                name
            )),
            name: name.to_string(),
            socket,
            max_energy_uj: Some(10_000_000),
        }
    }

    #[test]
    fn test_measure_phases_iterations_zero() {
        let config = Config {
            sockets: vec![0],
            json: false,
            csv: false,
            iterations: Some(0),
            jouleit_file: None,
            output_file: None,
            token_pattern: Some("_.*".to_string()),
            cmd: vec!["echo".to_string(), "test".to_string()],
        };

        let domains = vec![create_mock_domain("package-0", 0)];

        let result = measure_phases_iterations(&config, &domains, 0);
        assert!(result.is_err());

        if let Err(e) = result {
            let err = e.downcast::<JouleProfilerError>().unwrap();
            assert!(matches!(err, JouleProfilerError::InvalidIterations(0)));
        }
    }

    #[test]
    fn test_no_command() {
        let config = Config {
            sockets: vec![0],
            json: false,
            csv: false,
            iterations: None,
            jouleit_file: None,
            output_file: None,
            token_pattern: Some("_.*".to_string()),
            cmd: vec![],
        };

        let domains = vec![create_mock_domain("package-0", 0)];

        let result = measure_phases_once(&config, &domains);
        assert!(result.is_err());

        if let Err(e) = result {
            let err = e.downcast::<JouleProfilerError>().unwrap();
            assert!(matches!(err, JouleProfilerError::NoCommand));
        }
    }

    #[test]
    fn test_invalid_regex() {
        let config = Config {
            sockets: vec![0],
            json: false,
            csv: false,
            iterations: None,
            jouleit_file: None,
            output_file: None,
            token_pattern: Some("[invalid(".to_string()), // Invalid regex
            cmd: vec!["echo".to_string(), "test".to_string()],
        };

        let domains = vec![create_mock_domain("package-0", 0)];

        let result = measure_phases_once(&config, &domains);
        assert!(result.is_err());
    }
}
