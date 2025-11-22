use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

use anyhow::{Context, Result};
use log::{debug, error, info, trace, warn};

use crate::config::Config;
use crate::errors::JouleProfilerError;
use crate::measure::common::{
    PhaseMeasurement, PhasesResult, build_max_map, compute_measurement_from_snapshots,
};
use crate::rapl::{EnergySnapshot, RaplDomain, read_snapshot};

/// Measure one run in phases mode.
pub fn measure_phases_once(config: &Config, domains: &[RaplDomain]) -> Result<PhasesResult> {
    info!("Starting single phase measurement");

    if config.cmd.is_empty() {
        error!("No command specified for measurement");
        return Err(JouleProfilerError::NoCommand.into());
    }

    let token_start = config.token_start.as_ref().ok_or_else(|| {
        error!("token_start not configured for phases mode");
        JouleProfilerError::TokenNotFound("token_start not configured".to_string())
    })?;

    let token_end = config.token_end.as_ref().ok_or_else(|| {
        error!("token_end not configured for phases mode");
        JouleProfilerError::TokenNotFound("token_end not configured".to_string())
    })?;

    info!(
        "Running in phase mode with tokens start='{}', end='{}'",
        token_start, token_end
    );

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

    let mut work_start_snapshot: Option<EnergySnapshot> = None;
    let mut work_end_snapshot: Option<EnergySnapshot> = None;
    let mut line_count = 0;
    let mut start_token_count = 0;
    let mut end_token_count = 0;

    trace!("Starting to monitor command output for phase tokens");
    for line_res in reader.lines() {
        let line = line_res?;
        line_count += 1;

        if let Some(f) = out_file.as_mut() {
            writeln!(f, "{line}").map_err(|e| {
                error!("Failed to write to output file: {}", e);
                JouleProfilerError::OutputWriteFailed(e.to_string())
            })?;
        } else {
            println!("{line}");
        }

        if line.contains(token_start) {
            start_token_count += 1;

            if work_start_snapshot.is_none() {
                info!(
                    "✓ Detected start token '{}' at line {}",
                    token_start, line_count
                );
                let snap = read_snapshot(&filtered)?;
                debug!(
                    "Work phase start snapshot taken at {} µs",
                    snap.timestamp_us
                );
                work_start_snapshot = Some(snap);
            } else {
                warn!(
                    "⚠ Multiple occurrences of start token '{}' detected at line {} (already found at line {})",
                    token_start, line_count, start_token_count
                );
            }
        }

        if line.contains(token_end) {
            end_token_count += 1;

            if work_end_snapshot.is_none() {
                info!(
                    "✓ Detected end token '{}' at line {}",
                    token_end, line_count
                );
                let snap = read_snapshot(&filtered)?;
                debug!("Work phase end snapshot taken at {} µs", snap.timestamp_us);
                work_end_snapshot = Some(snap);
            } else {
                warn!(
                    "⚠ Multiple occurrences of end token '{}' detected at line {}",
                    token_end, line_count
                );
            }
        }
    }

    debug!("Processed {} lines of command output", line_count);

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

    if start_token_count > 1 {
        warn!(
            "⚠ Start token '{}' appeared {} times (expected exactly 1)",
            token_start, start_token_count
        );
    }

    if end_token_count > 1 {
        warn!(
            "⚠ End token '{}' appeared {} times (expected exactly 1)",
            token_end, end_token_count
        );
    }

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

    let mut phases = Vec::<PhaseMeasurement>::new();

    // Global (START -> END)
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
        result: global,
    });

    // Pre-work (START -> token_start)
    if let Some(ws) = &work_start_snapshot {
        debug!("Computing pre-work phase (START -> {})", token_start);
        let pre_duration_ms = duration_between_ms(&start_snapshot, ws);
        let pre = compute_measurement_from_snapshots(
            &filtered,
            &max_map,
            &start_snapshot,
            ws,
            pre_duration_ms,
            exit_code,
        )?;
        info!("Pre-work phase: {} ms", pre_duration_ms);
        phases.push(PhaseMeasurement {
            name: format!("pre_work (START -> {})", token_start),
            result: pre,
        });
    } else {
        warn!(
            "⚠ Start token '{}' was never detected in output; skipping pre-work phase",
            token_start
        );
    }

    // Work (token_start -> token_end)
    if let (Some(ws), Some(we)) = (&work_start_snapshot, &work_end_snapshot) {
        debug!("Computing work phase ({} -> {})", token_start, token_end);

        if we.timestamp_us < ws.timestamp_us {
            error!(
                "End token '{}' appeared before start token '{}' in timeline",
                token_end, token_start
            );
            return Err(JouleProfilerError::InvalidTokenOrder {
                start: token_start.clone(),
                end: token_end.clone(),
            }
            .into());
        }

        let work_duration_ms = duration_between_ms(ws, we);
        let work = compute_measurement_from_snapshots(
            &filtered,
            &max_map,
            ws,
            we,
            work_duration_ms,
            exit_code,
        )?;
        info!("Work phase: {} ms", work_duration_ms);
        phases.push(PhaseMeasurement {
            name: format!("work ({} -> {})", token_start, token_end),
            result: work,
        });
    } else {
        if work_start_snapshot.is_none() {
            warn!(
                "⚠ Start token '{}' not detected; skipping work phase",
                token_start
            );
        }
        if work_end_snapshot.is_none() {
            warn!(
                "⚠ End token '{}' not detected; skipping work phase",
                token_end
            );
        }
    }

    // Post-work (token_end -> END)
    if let Some(we) = &work_end_snapshot {
        debug!("Computing post-work phase ({} -> END)", token_end);
        let post_duration_ms = duration_between_ms(we, &end_snapshot);
        let post = compute_measurement_from_snapshots(
            &filtered,
            &max_map,
            we,
            &end_snapshot,
            post_duration_ms,
            exit_code,
        )?;
        info!("Post-work phase: {} ms", post_duration_ms);
        phases.push(PhaseMeasurement {
            name: format!("post_work ({} -> END)", token_end),
            result: post,
        });
    } else {
        warn!(
            "⚠ End token '{}' not detected; skipping post-work phase",
            token_end
        );
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
            token_start: Some("START".to_string()),
            token_end: Some("END".to_string()),
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
            token_start: Some("START".to_string()),
            token_end: Some("END".to_string()),
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
    fn test_no_domains() {
        let config = Config {
            sockets: vec![99],
            json: false,
            csv: false,
            iterations: None,
            jouleit_file: None,
            output_file: None,
            token_start: Some("START".to_string()),
            token_end: Some("END".to_string()),
            cmd: vec!["echo".to_string(), "test".to_string()],
        };

        let domains = vec![create_mock_domain("package-0", 0)];

        let result = measure_phases_once(&config, &domains);
        assert!(result.is_err());

        if let Err(e) = result {
            let err = e.downcast::<JouleProfilerError>().unwrap();
            assert!(matches!(err, JouleProfilerError::NoDomains));
        }
    }

    #[test]
    fn test_missing_token_start() {
        let config = Config {
            sockets: vec![0],
            json: false,
            csv: false,
            iterations: None,
            jouleit_file: None,
            output_file: None,
            token_start: None,
            token_end: Some("END".to_string()),
            cmd: vec!["echo".to_string(), "test".to_string()],
        };

        let domains = vec![create_mock_domain("package-0", 0)];

        let result = measure_phases_once(&config, &domains);
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_token_end() {
        let config = Config {
            sockets: vec![0],
            json: false,
            csv: false,
            iterations: None,
            jouleit_file: None,
            output_file: None,
            token_start: Some("START".to_string()),
            token_end: None,
            cmd: vec!["echo".to_string(), "test".to_string()],
        };

        let domains = vec![create_mock_domain("package-0", 0)];

        let result = measure_phases_once(&config, &domains);
        assert!(result.is_err());
    }
}
