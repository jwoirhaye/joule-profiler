use std::{
    env,
    fs::{File, OpenOptions, Permissions},
    os::unix::fs::{PermissionsExt, chown},
    path::PathBuf,
};

use anyhow::{Context, Result};

pub fn create_file_with_user_permissions(path: &str) -> Result<File> {
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&path)?;

    file.set_permissions(Permissions::from_mode(0o664))?;

    let uid: u32 = env::var("SUDO_UID")?
        .parse()
        .context("SUDO_UID is not a valid u32")?;

    let gid: u32 = env::var("SUDO_GID")?
        .parse()
        .context("SUDO_GID is not a valid u32")?;

    chown(&path, Some(uid), Some(gid))?;

    Ok(file)
}

pub fn get_absolute_path(filename: &str) -> Result<String> {
    let path = PathBuf::from(filename);
    let absolute_path = if path.is_absolute() {
        path
    } else {
        std::env::current_dir()?.join(&path)
    };

    Ok(absolute_path.display().to_string())
}
