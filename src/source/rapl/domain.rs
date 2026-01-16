use std::collections::HashSet;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::{env, fs};

use crate::error::JouleProfilerError;
use anyhow::Result;
use log::{debug, error, info, trace, warn};

/// Represents a RAPL (Running Average Power Limit) energy domain.
#[derive(Debug, Clone)]
pub struct RaplDomain {
    /// Path to the energy_uj file for reading current energy counter
    pub path: PathBuf,
    /// Logical name of the domain (e.g., "package-0", "core", "dram")
    pub name: String,
    /// CPU socket index this domain belongs to
    pub socket: u32,
    /// Maximum energy range in microjoules (for overflow detection)
    pub max_energy_uj: u64,
}

/// Checks if the operating system is Linux.
pub fn check_os() -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        Ok(())
    }

    #[cfg(not(target_os = "linux"))]
    {
        let os = std::env::consts::OS;
        Err(JouleProfilerError::UnsupportedOS(os.to_string()).into())
    }
}

/// Retrieve the RAPL domains on the machine, filtered with spec if one is provided.
pub fn get_domains(
    base_path: Option<&str>,
    spec: Option<&HashSet<u32>>,
) -> Result<Vec<RaplDomain>> {
    check_os()?;

    let base = rapl_base_path(base_path);
    check_rapl(&base)?;

    let domains = discover_domains(&base)?;
    let sockets = parse_or_all_sockets(&domains, spec);

    let filtered: Vec<RaplDomain> = domains
        .into_iter()
        .filter(|d| sockets.contains(&d.socket))
        .collect();

    Ok(filtered)
}

/// Checks if the RAPL interface is available at the given base path.
pub fn check_rapl(base: &str) -> Result<()> {
    debug!("Checking RAPL base path: {}", base);

    let path = Path::new(base);

    if !path.exists() {
        error!("RAPL path does not exist: {}", base);
        return Err(JouleProfilerError::RaplNotAvailable(base.into()).into());
    }

    if !path.is_dir() {
        error!("RAPL path is not a directory: {}", base);
        return Err(JouleProfilerError::InvalidRaplPath(base.into()).into());
    }

    if let Err(e) = fs::read_dir(path) {
        if e.kind() == std::io::ErrorKind::PermissionDenied {
            error!("Permission denied accessing RAPL path");
            return Err(JouleProfilerError::InsufficientPermissions.into());
        }
        return Err(e.into());
    }

    info!("RAPL interface found at {}", base);
    Ok(())
}

/// Discovers all available RAPL domains at the given base path.
pub fn discover_domains(base: &str) -> Result<Vec<RaplDomain>> {
    info!("Discovering RAPL domains in {}", base);

    let mut domains = Vec::new();

    let entries = fs::read_dir(base).map_err(|e| {
        error!("Failed to read RAPL base directory: {}", e);
        if e.kind() == std::io::ErrorKind::PermissionDenied {
            JouleProfilerError::InsufficientPermissions
        } else {
            JouleProfilerError::RaplReadError(format!("Failed to read {}: {}", base, e))
        }
    })?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if !path.is_dir() {
            trace!("Skipping non-directory {:?}", path);
            continue;
        }

        let Some(name) = path.file_name().and_then(OsStr::to_str) else {
            continue;
        };

        if !name.starts_with("intel-rapl:") {
            trace!("Skipping unrelated directory {}", name);
            continue;
        }

        add_domain_if_energy(&path, &mut domains)?;

        for sub in fs::read_dir(&path)? {
            let sub = sub?;
            let sub_path = sub.path();
            if sub_path.is_dir() {
                add_domain_if_energy(&sub_path, &mut domains)?;
            }
        }
    }

    if domains.is_empty() {
        warn!("No RAPL domains found");
        return Err(JouleProfilerError::NoDomains.into());
    }

    info!("Discovered {} RAPL domains", domains.len());
    Ok(domains)
}

/// Adds a RAPL domain to the output vector if it contains an energy_uj file.
fn add_domain_if_energy(dir: &Path, out: &mut Vec<RaplDomain>) -> Result<()> {
    let energy_path = dir.join("energy_uj");
    if !energy_path.exists() {
        trace!("No energy_uj in {:?}", dir);
        return Ok(());
    }

    let name = fs::read_to_string(dir.join("name"))
        .unwrap_or_else(|_| "unknown".to_string())
        .trim()
        .to_string();

    let socket = extract_socket_number(dir)?;

    let max_energy_uj_option = dir
        .join("max_energy_range_uj")
        .exists()
        .then(|| {
            fs::read_to_string(dir.join("max_energy_range_uj"))
                .ok()
                .and_then(|s| s.trim().parse::<u64>().ok())
        })
        .flatten();

    if let Some(max_energy_uj) = max_energy_uj_option {
        debug!(
            "Found domain: name={}, socket={}, max_energy_uj={}",
            name, socket, max_energy_uj
        );

        out.push(RaplDomain {
            path: energy_path,
            name,
            socket,
            max_energy_uj,
        });
    } else {
        warn!("Domain {:?} missing max_energy_range_uj", dir);
    }

    Ok(())
}

/// Extracts the socket number from a RAPL domain path.
fn extract_socket_number(path: &Path) -> Result<u32> {
    for comp in path.components() {
        if let std::path::Component::Normal(os) = comp
            && let Some(s) = os.to_str()
            && let Some(rest) = s.strip_prefix("intel-rapl:")
            && let Some(idx) = rest.split(':').next()
            && let Ok(n) = idx.parse::<u32>()
        {
            return Ok(n);
        }
    }
    Ok(0)
}

/// Discovers all unique socket indices from the given RAPL domains.
pub fn discover_sockets(domains: &[RaplDomain]) -> HashSet<u32> {
    domains.iter().map(|d| d.socket).collect()
}

/// Parses a socket specification string and validates against available domains.
pub fn filter_sockets(spec: &HashSet<u32>, sockets: &HashSet<u32>) -> HashSet<u32> {
    spec.intersection(sockets).cloned().collect()
}

/// Reads the current energy counter value from a RAPL domain.
pub fn read_energy(domain: &RaplDomain) -> Result<u64> {
    trace!("Reading energy for domain {}", domain.name);

    let content = fs::read_to_string(&domain.path).map_err(|e| {
        error!("Failed to read energy for {}: {}", domain.name, e);
        if e.kind() == std::io::ErrorKind::PermissionDenied {
            JouleProfilerError::InsufficientPermissions
        } else {
            JouleProfilerError::RaplReadError(format!("Failed to read {}: {}", domain.name, e))
        }
    })?;

    let energy = content.trim().parse::<u64>().map_err(|_| {
        error!(
            "Invalid energy value '{}' in {}",
            content.trim(),
            domain.name
        );
        JouleProfilerError::ParseEnergyError(format!(
            "Invalid energy value '{}' in domain {}",
            content.trim(),
            domain.name
        ))
    })?;

    trace!("Energy {} = {} µJ", domain.name, energy);
    Ok(energy)
}

/// Resolves the RAPL base path from configuration and environment.
pub fn rapl_base_path(config_override: Option<&str>) -> String {
    if let Some(path) = config_override {
        return path.to_string();
    }

    if let Ok(env_path) = env::var("JOULE_PROFILER_RAPL_PATH") {
        return env_path;
    }

    let default_path = "/sys/devices/virtual/powercap/intel-rapl";
    default_path.to_string()
}

/// Filter RAPL sockets with specified spec.
pub fn parse_or_all_sockets(domains: &[RaplDomain], spec: Option<&HashSet<u32>>) -> Vec<u32> {
    let mut sockets = discover_sockets(domains);
    if let Some(spec) = spec {
        sockets = filter_sockets(spec, &sockets);
    }
    sockets.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{create_dir_all, write};
    use tempfile::tempdir;

    fn make_domain_dir(
        base: &std::path::Path,
        name: &str,
        socket: u32,
        energy: u64,
        max_energy: u64,
    ) -> std::path::PathBuf {
        let dir = base.join(format!("intel-rapl:{}", socket));
        create_dir_all(&dir).unwrap();

        write(dir.join("name"), name).unwrap();
        write(dir.join("energy_uj"), energy.to_string()).unwrap();
        write(dir.join("max_energy_range_uj"), max_energy.to_string()).unwrap();

        dir
    }

    #[test]
    fn rapl_base_path_uses_override() {
        let path = rapl_base_path(Some("/custom/path"));
        assert_eq!(path, "/custom/path");
    }

    #[test]
    fn extract_socket_number_from_path() {
        let path = std::path::Path::new("/sys/devices/intel-rapl:2/intel-rapl:2:0");
        let socket = extract_socket_number(path).unwrap();
        assert_eq!(socket, 2);
    }

    #[test]
    fn extract_socket_number_defaults_to_zero() {
        let path = std::path::Path::new("/weird/path/no-socket");
        let socket = extract_socket_number(path).unwrap();
        assert_eq!(socket, 0);
    }

    #[test]
    fn discover_domains_finds_valid_domain() {
        let dir = tempdir().unwrap();
        let base = dir.path();

        make_domain_dir(base, "package", 0, 100, 1_000);

        let domains = discover_domains(base.to_str().unwrap()).unwrap();

        assert_eq!(domains.len(), 1);
        let d = &domains[0];
        assert_eq!(d.name, "package");
        assert_eq!(d.socket, 0);
        assert_eq!(d.max_energy_uj, 1_000);
    }

    #[test]
    fn discover_domains_ignores_missing_max_energy() {
        let dir = tempdir().unwrap();
        let base = dir.path();

        let domain = base.join("intel-rapl:0");
        create_dir_all(&domain).unwrap();
        write(domain.join("energy_uj"), "100").unwrap();

        let err = discover_domains(base.to_str().unwrap())
            .unwrap_err()
            .to_string();

        assert!(err.contains("No RAPL domains found"));
    }

    #[test]
    fn read_energy_reads_valid_value() {
        let dir = tempdir().unwrap();
        let energy_file = dir.path().join("energy_uj");
        write(&energy_file, "12345").unwrap();

        let domain = RaplDomain {
            path: energy_file,
            name: "package".to_string(),
            socket: 0,
            max_energy_uj: 1_000,
        };

        let energy = read_energy(&domain).unwrap();
        assert_eq!(energy, 12345);
    }

    #[test]
    fn read_energy_invalid_value_fails() {
        let dir = tempdir().unwrap();
        let energy_file = dir.path().join("energy_uj");
        write(&energy_file, "abc").unwrap();

        let domain = RaplDomain {
            path: energy_file,
            name: "package".to_string(),
            socket: 0,
            max_energy_uj: 1_000,
        };

        let err = read_energy(&domain).unwrap_err().to_string();
        assert!(err.contains("Invalid energy value"));
    }

    // ---------- sockets utilities ----------

    #[test]
    fn discover_sockets_returns_unique_sockets() {
        let domains = vec![
            RaplDomain {
                path: "a".into(),
                name: "pkg".into(),
                socket: 0,
                max_energy_uj: 1,
            },
            RaplDomain {
                path: "b".into(),
                name: "pkg".into(),
                socket: 1,
                max_energy_uj: 1,
            },
        ];

        let sockets = discover_sockets(&domains);
        assert_eq!(sockets.len(), 2);
        assert!(sockets.contains(&0));
        assert!(sockets.contains(&1));
    }

    #[test]
    fn filter_sockets_intersects_correctly() {
        let available: HashSet<u32> = [0, 1, 2].into_iter().collect();
        let spec: HashSet<u32> = [1, 3].into_iter().collect();

        let filtered = filter_sockets(&spec, &available);

        assert_eq!(filtered.len(), 1);
        assert!(filtered.contains(&1));
    }

    #[test]
    fn parse_or_all_sockets_without_spec_returns_all() {
        let domains = vec![
            RaplDomain {
                path: "a".into(),
                name: "pkg".into(),
                socket: 0,
                max_energy_uj: 1,
            },
            RaplDomain {
                path: "b".into(),
                name: "pkg".into(),
                socket: 1,
                max_energy_uj: 1,
            },
        ];

        let sockets = parse_or_all_sockets(&domains, None);
        assert_eq!(sockets.len(), 2);
    }
}
