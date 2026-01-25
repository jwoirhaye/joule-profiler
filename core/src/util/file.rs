use std::{
    env,
    fs::{File, OpenOptions, Permissions},
    io::ErrorKind,
    os::unix::fs::{PermissionsExt, chown},
    path::PathBuf,
};

const ROOT_UID_ENV_VAR: &str = "SUDO_UID";
const ROOT_GID_ENV_VAR: &str = "SUDO_GID";
const URW_GRW_OR_PERMS: u32 = 0o664;

pub fn create_file_with_user_permissions(path: &str) -> std::io::Result<File> {
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)?;

    file.set_permissions(Permissions::from_mode(URW_GRW_OR_PERMS))?;

    let uid_result = env::var(ROOT_UID_ENV_VAR)
        .map_err(|_| std::io::Error::new(ErrorKind::NotFound, "ROOT_UID not set"))?
        .parse::<u32>()
        .map_err(|_| std::io::Error::new(ErrorKind::InvalidData, "Invalid ROOT_UID"));

    let gid_result = env::var(ROOT_GID_ENV_VAR)
        .map_err(|_| std::io::Error::new(ErrorKind::NotFound, "ROOT_GID not set"))?
        .parse::<u32>()
        .map_err(|_| std::io::Error::new(ErrorKind::InvalidData, "Invalid ROOT_GID"));

    if let Ok(uid) = uid_result
        && let Ok(gid) = gid_result
    {
        chown(path, Some(uid), Some(gid))?;
    }

    Ok(file)
}

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
