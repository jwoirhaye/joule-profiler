pub mod domain;
pub mod snapshot;

pub use domain::{
    RaplDomain, check_os, check_rapl, discover_domains, discover_sockets, parse_sockets,
};
pub use snapshot::{EnergySnapshot, read_snapshot};

use log::{debug, info, trace};
use std::env;

/// Resolves the RAPL base path from configuration and environment.
pub fn rapl_base_path(config_override: Option<&String>) -> String {
    trace!("Resolving RAPL base path");

    if let Some(path) = config_override {
        info!("Using RAPL path from CLI override: {}", path);
        return path.clone();
    }

    if let Ok(env_path) = env::var("JOULE_PROFILER_RAPL_PATH") {
        info!(
            "Using RAPL path from environment variable JOULE_PROFILER_RAPL_PATH: {}",
            env_path
        );
        return env_path;
    }

    let default_path = "/sys/devices/virtual/powercap/intel-rapl";
    debug!("Using default RAPL path: {}", default_path);
    default_path.into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_rapl_base_path_default() {
        unsafe {
            env::remove_var("JOULE_PROFILER_RAPL_PATH");
        }

        let path = rapl_base_path(None);
        assert_eq!(path, "/sys/devices/virtual/powercap/intel-rapl");
    }

    #[test]
    #[serial]
    fn test_rapl_base_path_cli_override() {
        unsafe {
            env::remove_var("JOULE_PROFILER_RAPL_PATH");
        }

        let custom_path = "/custom/rapl/path".to_string();
        let path = rapl_base_path(Some(&custom_path));
        assert_eq!(path, "/custom/rapl/path");
    }

    #[test]
    #[serial]
    fn test_rapl_base_path_env_variable() {
        let env_path = "/env/rapl/path";
        unsafe {
            env::set_var("JOULE_PROFILER_RAPL_PATH", env_path);
        }

        let path = rapl_base_path(None);
        assert_eq!(path, env_path);

        unsafe {
            env::remove_var("JOULE_PROFILER_RAPL_PATH");
        }
    }

    #[test]
    #[serial]
    fn test_rapl_base_path_cli_takes_precedence_over_env() {
        let env_path = "/env/rapl/path";
        let cli_path = "/cli/rapl/path".to_string();

        unsafe {
            env::set_var("JOULE_PROFILER_RAPL_PATH", env_path);
        }

        let path = rapl_base_path(Some(&cli_path));
        assert_eq!(path, "/cli/rapl/path");

        unsafe {
            env::remove_var("JOULE_PROFILER_RAPL_PATH");
        }
    }

    #[test]
    #[serial]
    fn test_rapl_base_path_env_takes_precedence_over_default() {
        let env_path = "/env/rapl/path";
        unsafe {
            env::set_var("JOULE_PROFILER_RAPL_PATH", env_path);
        }

        let path = rapl_base_path(None);
        assert_eq!(path, env_path);
        assert_ne!(path, "/sys/devices/virtual/powercap/intel-rapl");

        unsafe {
            env::remove_var("JOULE_PROFILER_RAPL_PATH");
        }
    }

    #[test]
    #[serial]
    fn test_rapl_base_path_empty_string_override() {
        unsafe {
            env::remove_var("JOULE_PROFILER_RAPL_PATH");
        }

        let empty_path = "".to_string();
        let path = rapl_base_path(Some(&empty_path));
        assert_eq!(path, "");
    }

    #[test]
    #[serial]
    fn test_rapl_base_path_with_trailing_slash() {
        unsafe {
            env::remove_var("JOULE_PROFILER_RAPL_PATH");
        }

        let path_with_slash = "/custom/rapl/path/".to_string();
        let path = rapl_base_path(Some(&path_with_slash));
        assert_eq!(path, "/custom/rapl/path/");
    }

    #[test]
    #[serial]
    fn test_rapl_base_path_relative_path() {
        unsafe {
            env::remove_var("JOULE_PROFILER_RAPL_PATH");
        }

        let relative = "./relative/rapl".to_string();
        let path = rapl_base_path(Some(&relative));
        assert_eq!(path, "./relative/rapl");
    }
}
