use std::{
    env,
    fs::{File, OpenOptions, Permissions},
    os::unix::fs::{PermissionsExt, chown},
    path::PathBuf,
};

use anyhow::{Context, Result};

const ROOT_UID_ENV_VAR: &str = "SUDO_UID";
const ROOT_GID_ENV_VAR: &str = "SUDO_GID";
const URW_GRW_OR_PERMS: u32 = 0o664;

/// Create a file with user permissions in case of running with root permissions.
pub fn create_file_with_user_permissions(path: &str) -> Result<File> {
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)?;

    file.set_permissions(Permissions::from_mode(URW_GRW_OR_PERMS))?;

    let uid: u32 = env::var(ROOT_UID_ENV_VAR)?
        .parse()
        .context("Unable to parse root UID to u32")?;

    let gid: u32 = env::var(ROOT_GID_ENV_VAR)?
        .parse()
        .context("Unable to parse root GID to u32")?;

    chown(path, Some(uid), Some(gid))?;

    Ok(file)
}

/// Get the absolute path of a file.
pub fn get_absolute_path(filename: &str) -> Result<String> {
    let path = PathBuf::from(filename);
    let absolute_path = if path.is_absolute() {
        path
    } else {
        std::env::current_dir()?.join(path)
    };

    Ok(absolute_path.display().to_string())
}
