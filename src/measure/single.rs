use std::os::unix::process::ExitStatusExt;
use std::process::{Command, Stdio};
use std::time::Instant;

use anyhow::Result;
use log::{debug, error, info, trace, warn};

use crate::config::Config;
use crate::errors::JouleProfilerError;
use crate::measure::MeasurementResult;
use crate::source::metric::Metric;
use crate::source::{SourceManager};
use crate::util::file::create_file_with_user_permissions;

/// Performs a single measurement by executing the configured command.
pub fn measure_once(config: &Config, source_manager: &mut SourceManager) -> Result<MeasurementResult> {
    info!("Starting single measurement");

    if config.cmd.is_empty() {
        error!("No command specified for measurement");
        return Err(JouleProfilerError::NoCommand.into());
    }

    source_manager.measure()?;

    info!("Executing command: {:?}", config.cmd);
    let start_instant = Instant::now();

    let (exit_code, _status) = run_command(config)?;

    let elapsed = start_instant.elapsed();
    let duration_ms = elapsed.as_millis();

    if exit_code == 0 {
        info!(
            "Command completed successfully (duration: {:.3}s)",
            elapsed.as_secs_f64()
        );
    } else {
        warn!(
            "Command failed with exit code {} (duration: {:.3}s)",
            exit_code,
            elapsed.as_secs_f64()
        );
    }

    source_manager.measure()?;

    let mut metrics = Vec::new();

    for source in source_manager.retrieve()? {
        let source_metrics: Vec<Metric> = source
            .into_iter()
            .flat_map(|snapshot| snapshot.metrics)
            .collect();
        for metric in source_metrics {
            metrics.push(metric);
        }
    }

    info!("Measurement completed successfully");

    let result = MeasurementResult {
        metrics,
        duration_ms,
        exit_code,
    };

    Ok(result)
}

/// Executes the configured command and returns its exit code and status.
fn run_command(config: &Config) -> Result<(i32, std::process::ExitStatus)> {
    trace!("Preparing command execution");

    if config.cmd.is_empty() {
        error!("Attempted to run empty command");
        return Err(JouleProfilerError::NoCommand.into());
    }

    let mut command = Command::new(&config.cmd[0]);
    if config.cmd.len() > 1 {
        command.args(&config.cmd[1..]);
        debug!(
            "Command with {} argument(s): {:?}",
            config.cmd.len() - 1,
            &config.cmd[1..]
        );
    }

    if let Some(path) = &config.output_file {
        debug!("Redirecting stdout to file: {:?}", path);
        let file = create_file_with_user_permissions(path).map_err(|e| {
            error!("Failed to create output file {:?}: {}", path, e);
            JouleProfilerError::OutputFileCreationFailed(format!("{:?}: {}", path, e))
        })?;
        command.stdout(Stdio::from(file));
    } else {
        trace!("Using inherited stdout");
        command.stdout(Stdio::inherit());
    }

    command.stderr(Stdio::inherit());

    debug!("Spawning command: {}", config.cmd[0]);
    let status = command.status().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            error!("Command not found: {}", config.cmd[0]);
            JouleProfilerError::CommandNotFound(config.cmd[0].clone())
        } else {
            error!("Failed to execute command {:?}: {}", config.cmd, e);
            JouleProfilerError::CommandExecutionFailed(e.to_string())
        }
    })?;

    let exit_code = status.code().unwrap_or_else(|| {
        if let Some(signal) = status.signal() {
            warn!("Command killed by signal {}, using exit code 1", signal);
        } else {
            warn!("Command terminated without exit code, defaulting to 1");
        }
        1
    });

    trace!("Command exited with code: {}", exit_code);

    Ok((exit_code, status))
}

#[cfg(test)]
mod tests {
    use crate::config::OutputFormat;

    use super::*;
    // use std::path::PathBuf;

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

    fn create_test_config(cmd: Vec<String>) -> Config {
        Config {
            output_format: OutputFormat::Terminal,
            iterations: None,
            jouleit_file: None,
            output_file: None,
            token_pattern: None,
            cmd,
        }
    }

    // #[test]
    // fn test_no_command() {
    //     let config = create_test_config(vec![]);
    //     let domains = vec![create_mock_domain("package-0", 0)];

    //     let result = measure_once(&config, &domains);
    //     assert!(result.is_err());

    //     if let Err(e) = result {
    //         let err = e.downcast::<JouleProfilerError>().unwrap();
    //         assert!(matches!(err, JouleProfilerError::NoCommand));
    //     }
    // }

    // #[test]
    // fn test_no_domains_for_sockets() {
    //     let config = create_test_config(vec!["echo".to_string(), "test".to_string()]);
    //     let domains = vec![create_mock_domain("package-0", 0)];

    //     let result = measure_once(&config, &domains);
    //     assert!(result.is_err());

    //     if let Err(e) = result {
    //         let err = e.downcast::<JouleProfilerError>().unwrap();
    //         assert!(matches!(err, JouleProfilerError::NoDomains));
    //     }
    // }

    #[test]
    fn test_run_command_empty() {
        let config = create_test_config(vec![]);

        let result = run_command(&config);
        assert!(result.is_err());

        if let Err(e) = result {
            let err = e.downcast::<JouleProfilerError>().unwrap();
            assert!(matches!(err, JouleProfilerError::NoCommand));
        }
    }

    #[test]
    fn test_run_command_not_found() {
        let config = create_test_config(vec![
            "this-command-definitely-does-not-exist-12345".to_string(),
        ]);

        let result = run_command(&config);
        assert!(result.is_err());

        if let Err(e) = result {
            let err = e.downcast::<JouleProfilerError>().unwrap();
            assert!(
                matches!(err, JouleProfilerError::CommandNotFound(ref cmd) if cmd == "this-command-definitely-does-not-exist-12345")
            );
        }
    }

    #[test]
    fn test_run_command_success() {
        let config = create_test_config(vec!["echo".to_string(), "test".to_string()]);

        let result = run_command(&config);
        assert!(result.is_ok());

        let (exit_code, status) = result.unwrap();
        assert_eq!(exit_code, 0);
        assert!(status.success());
    }

    #[test]
    fn test_run_command_with_failure() {
        let config = create_test_config(vec!["false".to_string()]);

        let result = run_command(&config);
        assert!(result.is_ok());

        let (exit_code, status) = result.unwrap();
        assert_ne!(exit_code, 0);
        assert!(!status.success());
    }

    #[test]
    fn test_run_command_with_args() {
        let config = create_test_config(vec![
            "echo".to_string(),
            "hello".to_string(),
            "world".to_string(),
        ]);

        let result = run_command(&config);
        assert!(result.is_ok());

        let (exit_code, _) = result.unwrap();
        assert_eq!(exit_code, 0);
    }

    #[test]
    #[cfg(unix)]
    fn test_run_command_with_signal() {
        use std::process::{Child, Command as StdCommand};
        use std::thread;
        use std::time::Duration;

        let config = create_test_config(vec!["sleep".to_string(), "60".to_string()]);

        let mut child: Child = StdCommand::new(&config.cmd[0])
            .args(&config.cmd[1..])
            .spawn()
            .unwrap();

        thread::sleep(Duration::from_millis(100));

        child.kill().unwrap();
        let status = child.wait().unwrap();

        let exit_code = status.code().unwrap_or(1);
        assert_ne!(exit_code, 0);
    }
}
