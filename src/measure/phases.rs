use std::fmt::Display;
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};
use log::{debug, error, info, trace, warn};
use regex::Regex;

use crate::config::Config;
use crate::errors::JouleProfilerError;
use crate::measure::{PhaseMeasurement, PhasesResult};
use crate::source::MetricSource;
use crate::source::metric::Snapshot;

use crate::util::file::create_file_with_user_permissions;
use crate::util::get_timestamp;

#[derive(Debug, Clone)]
pub enum PhaseToken {
    Start,
    Token(String),
    End,
}

impl Display for PhaseToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PhaseToken::Start => f.write_str("START"),
            PhaseToken::Token(token) => f.write_str(token),
            PhaseToken::End => f.write_str("END"),
        }
    }
}

impl From<PhaseToken> for Option<String> {
    fn from(token: PhaseToken) -> Self {
        match token {
            PhaseToken::Start | PhaseToken::End => None,
            PhaseToken::Token(token) => Some(token),
        }
    }
}

/// Detected token with timestamp
#[derive(Debug, Clone)]
struct Phase {
    token: PhaseToken,
    snapshot_index: usize,
    timestamp: u128,
    line_number: Option<usize>,
}

/// Measure one run in phases mode with regex pattern.
pub fn measure_phases_once(config: &Config, sources: &mut [MetricSource]) -> Result<PhasesResult> {
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

    let mut phases = Vec::new();

    let mut nb_snapshots = 0;

    let start_timestamp = get_timestamp();
    phases.push(Phase {
        token: PhaseToken::Start,
        timestamp: start_timestamp,
        snapshot_index: nb_snapshots,
        line_number: None,
    });

    nb_snapshots += 1;
    for source in sources.iter_mut() {
        source.measure()?;
    }

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

    let mut out_file: Option<File> = if let Some(path) = &config.output_file {
        debug!("Creating output file: {:?}", path);
        let file = create_file_with_user_permissions(path).map_err(|e| {
            error!("Failed to create output file {:?}: {}", path, e);
            JouleProfilerError::OutputFileCreationFailed(format!("{:?}: {}", path, e))
        })?;
        Some(file)
    } else {
        debug!("No output file specified, using stdout");
        None
    };

    trace!(
        "Starting to monitor command output for tokens matching pattern '{}'",
        token_pattern
    );

    for (line_number, line_res) in reader.lines().enumerate() {
        let line = match line_res {
            Ok(l) => l,
            Err(e) if e.kind() == io::ErrorKind::InvalidData => {
                warn!(
                    "Non-UTF8 data in child stdout at line {} — skipping this line",
                    line_number + 1
                );
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

            info!("✓ Detected token '{}' at line {}", token, line_number + 1);

            let phase_timestamp = get_timestamp();

            for source in sources.iter_mut() {
                source.measure()?;
            }

            phases.push(Phase {
                token: PhaseToken::Token(token),
                timestamp: phase_timestamp,
                snapshot_index: nb_snapshots,
                line_number: Some(line_number + 1),
            });
            nb_snapshots += 1;
        }
    }

    info!("Found {} phase", phases.len());

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

    for source in sources.iter_mut() {
        source.measure()?;
    }

    let end_timestamp = get_timestamp();
    phases.push(Phase {
        token: PhaseToken::End,
        timestamp: end_timestamp,
        snapshot_index: nb_snapshots,
        line_number: None,
    });

    let mut phases_metrics = Vec::with_capacity(phases.len());
    let mut sources_phases: Vec<Vec<Snapshot>> = Vec::with_capacity(sources.len());

    for source in sources {
        sources_phases.push(source.retrieve()?)
    }

    for phase in &phases[0..nb_snapshots] {
        let mut phase_metrics = Vec::new();
        for source_snapshots in &sources_phases {
            phase_metrics.extend(source_snapshots[phase.snapshot_index].metrics.clone());
        }
        phases_metrics.push(phase_metrics);
    }

    let mut phases_measurements = Vec::new();

    for (i, phases) in phases.windows(2).enumerate() {
        let (begin_phase, end_phase) = (&phases[0], &phases[1]);
        let metrics = phases_metrics[i].clone();
        let duration_ms = end_phase.timestamp - begin_phase.timestamp;

        let phase_mesurement = PhaseMeasurement::new(
            &begin_phase.token,
            &end_phase.token,
            begin_phase.line_number,
            end_phase.line_number,
            metrics,
            duration_ms,
        );
        phases_measurements.push(phase_mesurement);
    }

    info!(
        "Phase measurement completed: {} phase(s) computed",
        phases.len()
    );

    Ok(PhasesResult {
        phases: phases_measurements,
    })
}

/// Run phases measurement N times.
pub fn measure_phases_iterations(
    config: &Config,
    sources: &mut [MetricSource],
    iterations: usize,
) -> Result<Vec<(usize, PhasesResult)>> {
    if iterations == 0 {
        return Err(JouleProfilerError::InvalidIterations(0).into());
    }

    info!("Starting {} phase measurement iteration(s)", iterations);

    let mut all = Vec::with_capacity(iterations);

    for i in 0..iterations {
        info!("═══ Phase iteration {}/{} ═══", i + 1, iterations);

        match measure_phases_once(config, sources) {
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

    // fn create_mock_domain(name: &str, socket: u32) -> RaplDomain {
    //     RaplDomain {
    //         path: PathBuf::from(format!(
    //             "/sys/class/powercap/intel-rapl:0/{}/energy_uj",
    //             name
    //         )),
    //         name: name.to_string(),
    //         socket,
    //         max_energy_uj: Some(10_000_000),
    //     }
    // }

    // #[test]
    // fn test_measure_phases_iterations_zero() {
    //     let config = Config {
    //         json: false,
    //         csv: false,
    //         iterations: Some(0),
    //         jouleit_file: None,
    //         output_file: None,
    //         token_pattern: Some("_.*".to_string()),
    //         cmd: vec!["echo".to_string(), "test".to_string()],
    //     };

    //     let domains = vec![create_mock_domain("package-0", 0)];

    //     let result = measure_phases_iterations(&config, sources, 0);
    //     assert!(result.is_err());

    //     if let Err(e) = result {
    //         let err = e.downcast::<JouleProfilerError>().unwrap();
    //         assert!(matches!(err, JouleProfilerError::InvalidIterations(0)));
    //     }
    // }

    // #[test]
    // fn test_no_command() {
    //     let config = Config {
    //         json: false,
    //         csv: false,
    //         iterations: None,
    //         jouleit_file: None,
    //         output_file: None,
    //         token_pattern: Some("_.*".to_string()),
    //         cmd: vec![],
    //     };

    //     let domains = vec![create_mock_domain("package-0", 0)];

    //     let result = measure_phases_once(&config, &domains);
    //     assert!(result.is_err());

    //     if let Err(e) = result {
    //         let err = e.downcast::<JouleProfilerError>().unwrap();
    //         assert!(matches!(err, JouleProfilerError::NoCommand));
    //     }
    // }

    // #[test]
    // fn test_invalid_regex() {
    //     let config = Config {
    //         json: false,
    //         csv: false,
    //         iterations: None,
    //         jouleit_file: None,
    //         output_file: None,
    //         token_pattern: Some("[invalid(".to_string()), // Invalid regex
    //         cmd: vec!["echo".to_string(), "test".to_string()],
    //     };

    //     let domains = vec![create_mock_domain("package-0", 0)];

    //     let result = measure_phases_once(&config, &domains);
    //     assert!(result.is_err());
    // }
}
