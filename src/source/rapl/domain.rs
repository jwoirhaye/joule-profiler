use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use log::{debug, error, info, trace, warn};

use crate::errors::JouleProfilerError;

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
    pub max_energy_uj: Option<u64>,
}

/// Checks if the operating system is Linux.
pub fn check_os() -> Result<()> {
    debug!("Checking operating system compatibility");

    #[cfg(target_os = "linux")]
    {
        info!("Operating system check passed: Linux detected");
        Ok(())
    }

    #[cfg(not(target_os = "linux"))]
    {
        let os = std::env::consts::OS;
        error!("Operating system check failed: {} is not supported", os);
        Err(JouleProfilerError::UnsupportedOS(os.to_string()).into())
    }
}

/// Checks if the RAPL interface is available at the given base path.
pub fn check_rapl(base: &str) -> Result<()> {
    debug!("Checking RAPL availability at path: {}", base);
    let path = Path::new(base);

    if !path.exists() {
        error!("RAPL interface not available at path: {}", base);
        return Err(JouleProfilerError::RaplNotAvailable(base.into()).into());
    }

    if !path.is_dir() {
        error!("RAPL path exists but is not a directory: {}", base);
        return Err(JouleProfilerError::InvalidRaplPath(base.into()).into());
    }

    if let Err(e) = fs::read_dir(path) {
        if e.kind() == std::io::ErrorKind::PermissionDenied {
            error!("Insufficient permissions to access RAPL at: {}", base);
            return Err(JouleProfilerError::InsufficientPermissions.into());
        }
        return Err(e.into());
    }

    info!("RAPL interface found at: {}", base);
    Ok(())
}

/// Discovers all available RAPL domains at the given base path.
pub fn discover_domains(base: &str) -> Result<Vec<RaplDomain>> {
    info!("Starting RAPL domain discovery at: {}", base);
    let mut domains = Vec::new();

    let entries = fs::read_dir(base).map_err(|e| {
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
            trace!("Skipping non-directory entry: {:?}", path);
            continue;
        }

        let Some(name) = path.file_name().and_then(OsStr::to_str) else {
            trace!("Skipping entry with invalid filename: {:?}", path);
            continue;
        };

        if !name.starts_with("intel-rapl:") {
            trace!("Skipping non-RAPL directory: {}", name);
            continue;
        }

        debug!("Examining RAPL directory: {}", name);
        add_domain_if_energy(&path, &mut domains)?;

        for sub in fs::read_dir(&path)? {
            let sub = sub?;
            let sub_path = sub.path();
            if sub_path.is_dir() {
                trace!("Examining subdomain: {:?}", sub_path);
                add_domain_if_energy(&sub_path, &mut domains)?;
            }
        }
    }

    if domains.is_empty() {
        error!("No RAPL domains found in: {}", base);
        return Err(JouleProfilerError::NoDomains.into());
    }

    info!(
        "Successfully discovered {} RAPL domain(s): [{}]",
        domains.len(),
        domains
            .iter()
            .map(|d| format!("{} (socket {})", d.name, d.socket))
            .collect::<Vec<_>>()
            .join(", ")
    );
    Ok(domains)
}

/// Adds a RAPL domain to the output vector if it contains an energy_uj file.
fn add_domain_if_energy(dir: &Path, out: &mut Vec<RaplDomain>) -> Result<()> {
    let energy_path = dir.join("energy_uj");
    if !energy_path.exists() {
        trace!("No energy_uj file found in: {:?}", dir);
        return Ok(());
    }

    let name = fs::read_to_string(dir.join("name"))
        .map_err(|e| {
            warn!("Failed to read domain name from {:?}: {}", dir, e);
            e
        })
        .unwrap_or_else(|_| "unknown".to_string())
        .trim()
        .to_string();

    let socket = extract_socket_number(dir)?;

    let max_energy_uj = dir
        .join("max_energy_range_uj")
        .exists()
        .then(|| {
            fs::read_to_string(dir.join("max_energy_range_uj"))
                .ok()
                .and_then(|s| s.trim().parse::<u64>().ok())
        })
        .flatten();

    debug!(
        "Added RAPL domain: {} (socket {}, max_energy: {:?})",
        name, socket, max_energy_uj
    );

    out.push(RaplDomain {
        path: energy_path,
        name,
        socket,
        max_energy_uj,
    });

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
            trace!("Extracted socket number {} from path: {:?}", n, path);
            return Ok(n);
        }
    }
    warn!(
        "Failed to extract socket number from path: {:?}, defaulting to 0",
        path
    );
    Ok(0)
}

/// Discovers all unique socket indices from the given RAPL domains.
pub fn discover_sockets(domains: &[RaplDomain]) -> Vec<u32> {
    let mut v: Vec<u32> = domains.iter().map(|d| d.socket).collect();
    v.sort_unstable();
    v.dedup();
    debug!("Discovered sockets: {:?}", v);
    v
}

/// Parses a socket specification string and validates against available domains.
pub fn parse_sockets(spec: &str, domains: &[RaplDomain]) -> Result<Vec<u32>> {
    debug!("Parsing socket specification: '{}'", spec);
    let all = discover_sockets(domains);
    let mut res = Vec::new();

    for part in spec.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }

        let s: u32 = part.parse().map_err(|_| {
            error!(
                "Invalid socket specification: '{}' is not a valid number",
                part
            );
            JouleProfilerError::InvalidSocketSpec(part.to_string())
        })?;

        if !all.contains(&s) {
            error!("Socket {} not found in available sockets {:?}", s, all);
            return Err(JouleProfilerError::SocketNotFound(s).into());
        }

        trace!("Added socket {} to selection", s);
        res.push(s);
    }

    if res.is_empty() {
        warn!("Empty socket specification, using all available sockets");
        return Ok(all);
    }

    res.sort_unstable();
    res.dedup();
    info!("Selected sockets: {:?}", res);
    Ok(res)
}

/// Reads the current energy counter value from a RAPL domain.
pub fn read_energy(domain: &RaplDomain) -> Result<u64> {
    trace!("Reading energy from: {:?}", domain.path);

    let content = fs::read_to_string(&domain.path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::PermissionDenied {
            error!("Permission denied reading energy from: {:?}", domain.path);
            JouleProfilerError::InsufficientPermissions
        } else {
            error!("Failed to read energy from {:?}: {}", domain.path, e);
            JouleProfilerError::RaplReadError(format!("Failed to read {}: {}", domain.name, e))
        }
    })?;

    let energy = content.trim().parse::<u64>().map_err(|e| {
        error!(
            "Failed to parse energy value '{}' from {:?}: {}",
            content.trim(),
            domain.path,
            e
        );
        JouleProfilerError::ParseEnergyError(format!(
            "Invalid energy value '{}' in domain {}",
            content.trim(),
            domain.name
        ))
    })?;

    trace!(
        "Read energy value: {} µJ from domain {}",
        energy, domain.name
    );
    Ok(energy)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_os_on_linux() {
        #[cfg(target_os = "linux")]
        {
            let result = check_os();
            assert!(result.is_ok());
        }

        #[cfg(not(target_os = "linux"))]
        {
            let result = check_os();
            assert!(result.is_err());

            if let Err(e) = result {
                let err = e.downcast::<JouleProfilerError>().unwrap();
                assert!(matches!(err, JouleProfilerError::UnsupportedOS(_)));
            }
        }
    }

    #[test]
    fn test_discover_sockets_empty() {
        let domains = vec![];
        let sockets = discover_sockets(&domains);
        assert!(sockets.is_empty());
    }

    #[test]
    fn test_discover_sockets_single() {
        let domains = vec![RaplDomain {
            path: PathBuf::from("/test/energy_uj"),
            name: "package-0".to_string(),
            socket: 0,
            max_energy_uj: None,
        }];

        let sockets = discover_sockets(&domains);
        assert_eq!(sockets, vec![0]);
    }

    #[test]
    fn test_discover_sockets_multiple() {
        let domains = vec![
            RaplDomain {
                path: PathBuf::from("/test/0/energy_uj"),
                name: "package-0".to_string(),
                socket: 0,
                max_energy_uj: None,
            },
            RaplDomain {
                path: PathBuf::from("/test/1/energy_uj"),
                name: "package-1".to_string(),
                socket: 1,
                max_energy_uj: None,
            },
            RaplDomain {
                path: PathBuf::from("/test/0/core/energy_uj"),
                name: "core".to_string(),
                socket: 0,
                max_energy_uj: None,
            },
        ];

        let sockets = discover_sockets(&domains);
        assert_eq!(sockets, vec![0, 1]);
    }

    #[test]
    fn test_parse_sockets_empty_returns_all() {
        let domains = vec![
            RaplDomain {
                path: PathBuf::from("/test/0/energy_uj"),
                name: "package-0".to_string(),
                socket: 0,
                max_energy_uj: None,
            },
            RaplDomain {
                path: PathBuf::from("/test/1/energy_uj"),
                name: "package-1".to_string(),
                socket: 1,
                max_energy_uj: None,
            },
        ];

        let result = parse_sockets("", &domains).unwrap();
        assert_eq!(result, vec![0, 1]);
    }

    #[test]
    fn test_parse_sockets_single() {
        let domains = vec![
            RaplDomain {
                path: PathBuf::from("/test/0/energy_uj"),
                name: "package-0".to_string(),
                socket: 0,
                max_energy_uj: None,
            },
            RaplDomain {
                path: PathBuf::from("/test/1/energy_uj"),
                name: "package-1".to_string(),
                socket: 1,
                max_energy_uj: None,
            },
        ];

        let result = parse_sockets("0", &domains).unwrap();
        assert_eq!(result, vec![0]);
    }

    #[test]
    fn test_parse_sockets_multiple() {
        let domains = vec![
            RaplDomain {
                path: PathBuf::from("/test/0/energy_uj"),
                name: "package-0".to_string(),
                socket: 0,
                max_energy_uj: None,
            },
            RaplDomain {
                path: PathBuf::from("/test/1/energy_uj"),
                name: "package-1".to_string(),
                socket: 1,
                max_energy_uj: None,
            },
            RaplDomain {
                path: PathBuf::from("/test/2/energy_uj"),
                name: "package-2".to_string(),
                socket: 2,
                max_energy_uj: None,
            },
        ];

        let result = parse_sockets("0,2", &domains).unwrap();
        assert_eq!(result, vec![0, 2]);
    }

    #[test]
    fn test_parse_sockets_with_spaces() {
        let domains = vec![
            RaplDomain {
                path: PathBuf::from("/test/0/energy_uj"),
                name: "package-0".to_string(),
                socket: 0,
                max_energy_uj: None,
            },
            RaplDomain {
                path: PathBuf::from("/test/1/energy_uj"),
                name: "package-1".to_string(),
                socket: 1,
                max_energy_uj: None,
            },
        ];

        let result = parse_sockets(" 0 , 1 ", &domains).unwrap();
        assert_eq!(result, vec![0, 1]);
    }

    #[test]
    fn test_parse_sockets_invalid_number() {
        let domains = vec![RaplDomain {
            path: PathBuf::from("/test/0/energy_uj"),
            name: "package-0".to_string(),
            socket: 0,
            max_energy_uj: None,
        }];

        let result = parse_sockets("abc", &domains);
        assert!(result.is_err());

        if let Err(e) = result {
            let err = e.downcast::<JouleProfilerError>().unwrap();
            assert!(matches!(err, JouleProfilerError::InvalidSocketSpec(_)));
        }
    }

    #[test]
    fn test_parse_sockets_not_found() {
        let domains = vec![RaplDomain {
            path: PathBuf::from("/test/0/energy_uj"),
            name: "package-0".to_string(),
            socket: 0,
            max_energy_uj: None,
        }];

        let result = parse_sockets("99", &domains);
        assert!(result.is_err());

        if let Err(e) = result {
            let err = e.downcast::<JouleProfilerError>().unwrap();
            assert!(matches!(err, JouleProfilerError::SocketNotFound(99)));
        }
    }

    #[test]
    fn test_parse_sockets_deduplicated() {
        let domains = vec![
            RaplDomain {
                path: PathBuf::from("/test/0/energy_uj"),
                name: "package-0".to_string(),
                socket: 0,
                max_energy_uj: None,
            },
            RaplDomain {
                path: PathBuf::from("/test/1/energy_uj"),
                name: "package-1".to_string(),
                socket: 1,
                max_energy_uj: None,
            },
        ];

        let result = parse_sockets("0,1,0,1", &domains).unwrap();
        assert_eq!(result, vec![0, 1]);
    }
}
