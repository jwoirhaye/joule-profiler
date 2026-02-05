use std::{
    env,
    fs::{File, OpenOptions, Permissions},
    os::unix::fs::{PermissionsExt, chown},
    path::PathBuf,
};

use crate::util::{sys::is_root, time::get_timestamp_millis};

/// Get the absolute path of a file.
pub fn get_absolute_path(filename: &str) -> Result<String, std::io::Error> {
    let path = PathBuf::from(filename);
    let absolute_path = if path.is_absolute() {
        path
    } else {
        env::current_dir()?.join(path)
    };

    Ok(absolute_path.display().to_string())
}

/// Generates a default filename for iteration data.
pub fn default_iterations_filename(ext: &str) -> String {
    format!("data{}.{}", get_timestamp_millis(), ext)
}

const ROOT_UID_ENV_VAR: &str = "SUDO_UID";
const ROOT_GID_ENV_VAR: &str = "SUDO_GID";
const URW_GRW_OR_PERMS: u32 = 0o664;

/// Creates a file with user permissions even if the current user is root.
pub fn create_file_with_user_permissions(path: &str) -> std::io::Result<File> {
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)?;

    if is_root() {
        file.set_permissions(Permissions::from_mode(URW_GRW_OR_PERMS))?;

        let uid = env::var(ROOT_UID_ENV_VAR)
            .ok()
            .and_then(|v| v.parse::<u32>().ok());

        let gid = env::var(ROOT_GID_ENV_VAR)
            .ok()
            .and_then(|v| v.parse::<u32>().ok());

        chown(path, uid, gid)?;
    }

    Ok(file)
}
