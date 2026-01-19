use std::process::{Command, ExitStatus, Stdio};

use crate::util::file::create_file_with_user_permissions;

/// Executes the configured command and returns its exit code and status.
pub fn run_command(
    cmd: &[String],
    output_file: Option<&String>,
) -> Result<(i32, ExitStatus), std::io::Error> {
    let mut command = Command::new(&cmd[0]);
    command.args(&cmd[1..]);

    if let Some(path) = &output_file {
        let file = create_file_with_user_permissions(path)?;
        command.stdout(Stdio::from(file));
    } else {
        command.stdout(Stdio::inherit());
    }

    command.stderr(Stdio::inherit());

    let status = command.status()?;
    let exit_code = status.code().unwrap_or(1);

    Ok((exit_code, status))
}
