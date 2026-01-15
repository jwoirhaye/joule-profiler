use std::process::{Command, ExitStatus, Stdio};

use anyhow::Result;

use crate::{error::JouleProfilerError, util::file::create_file_with_user_permissions};

pub mod list_sensors;
pub mod phases;
pub mod simple;

/// Executes the configured command and returns its exit code and status.
pub fn run_command(cmd: &[String], output_file: Option<&String>) -> Result<(i32, ExitStatus)> {
    if cmd.is_empty() {
        return Err(JouleProfilerError::NoCommand.into());
    }

    let mut command = Command::new(&cmd[0]);
    command.args(&cmd[1..]);

    if let Some(path) = &output_file {
        let file = create_file_with_user_permissions(path).map_err(|e| {
            JouleProfilerError::OutputFileCreationFailed(format!("{:?}: {}", path, e))
        })?;
        command.stdout(Stdio::from(file));
    } else {
        command.stdout(Stdio::inherit());
    }

    command.stderr(Stdio::inherit());

    let status = command.status().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            JouleProfilerError::CommandNotFound(cmd[0].clone())
        } else {
            JouleProfilerError::CommandExecutionFailed(e.to_string())
        }
    })?;

    let exit_code = status.code().unwrap_or(1);

    Ok((exit_code, status))
}
