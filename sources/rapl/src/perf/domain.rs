use std::{
    collections::HashSet,
    fs::{self, File},
    io::Read,
    os::fd::{FromRawFd, RawFd},
};

use log::{debug, error, info, trace, warn};
use perf_event_open_sys::{bindings::perf_event_attr, perf_event_open};

use crate::{
    Result,
    domain_type::RaplDomainType,
    error::RaplError,
    perf::{PERF_RAPL_PATH, socket::discover_socket_topology},
};

/// Flag used for `perf_event_open` to automatically close the FD on exec.
const PERF_FLAG_FD_CLOEXEC: u64 = 8;

/// Represents a RAPL domain with its perf_event counter.
///
/// Each domain corresponds to a measurable component like `PACKAGE-0`, `DRAM-0`, etc.
#[derive(Debug)]
pub struct PerfRaplDomain {
    /// Domain type (package, dram, core, etc.).
    pub domain_type: RaplDomainType,
    /// Socket ID where this domain resides.
    pub socket: u32,
    /// Scaling factor to convert raw counter to joules.
    pub scale: f64,
    /// File descriptor for the perf counter.
    pub fd: File,
}

impl PerfRaplDomain {
    /// Create a new `PerfRaplDomain`.
    pub fn new(domain_type: RaplDomainType, socket: u32, scale: f64, fd: File) -> Self {
        Self {
            domain_type,
            socket,
            scale,
            fd,
        }
    }

    /// Read the raw perf counter value from the file descriptor.
    ///
    /// # Returns
    ///
    /// The current counter value as `u64`.
    pub fn read_counter(&mut self) -> Result<u64> {
        let mut buf = [0u8; 8];
        self.fd.read_exact(&mut buf)?;
        Ok(u64::from_ne_bytes(buf))
    }

    /// Apply the domain-specific scaling factor to convert raw value to joules.
    pub fn compute_scale(&self, value: u64) -> f64 {
        value as f64 * self.scale
    }

    /// Return a human-readable domain name including socket, e.g., `PACKAGE-0`.
    pub fn get_name(&self) -> String {
        self.domain_type.to_string_socket(self.socket)
    }
}

/// Discover available RAPL domains and open perf counters for each.
///
/// # Arguments
///
/// * `pmu_type` - PMU type obtained from sysfs.
/// * `rapl_path` - Path to RAPL sysfs directory.
/// * `domains_to_discover` - Optional filter for socket IDs.
///
/// # Returns
///
/// A vector of initialized `PerfRaplDomain` instances.
///
/// # Errors
///
/// Returns `RaplError` if any perf counter cannot be opened or parsed.
pub fn discover_domains_and_open_counters(
    pmu_type: u32,
    rapl_path: &str,
    domains_to_discover: Option<&HashSet<u32>>,
) -> Result<Vec<PerfRaplDomain>> {
    let mut domains = Vec::new();
    let domains_dir_path = format!("{}/events", rapl_path);

    info!("Discovering RAPL domains under {}", domains_dir_path);

    let socket_topology = discover_socket_topology(domains_to_discover)?;
    debug!("Socket topology: {:?}", socket_topology);

    for socket in &socket_topology {
        debug!(
            "Processing socket {} (cpus={:?})",
            socket.socket_id, socket.cpus_id
        );

        for entry_result in fs::read_dir(&domains_dir_path)? {
            let entry = entry_result?;
            let file_name = entry.file_name().into_string().map_err(|_| {
                let entry_name = entry.file_name().into_string().map_err(|err| {
                    RaplError::RaplReadError(format!("Cannot parse dir entry {:?}", err))
                });
                match entry_name {
                    Ok(name) => RaplError::UnknownDomain(name),
                    Err(err) => err,
                }
            })?;

            if file_name.ends_with(".scale")
                || file_name.ends_with(".unit")
                || entry.file_type()?.is_dir()
            {
                continue;
            }

            debug!("Found RAPL domain file {}", file_name);

            let event = read_domain_event_number(&file_name)?;
            let scale = read_domain_scale(&file_name)?;
            let domain_type = file_name.try_into()?;

            let first_cpu_socket = if let Some(first_socket_cpu) = socket.cpus_id.first() {
                first_socket_cpu
            } else {
                warn!(
                    "Socket {} has no CPUs, skipping domain {}",
                    socket.socket_id, domain_type
                );
                continue;
            };

            trace!(
                "Opening counter: socket={}, cpu={}, domain={}, event={}",
                socket.socket_id, first_cpu_socket, domain_type, event
            );

            let fd = open_domain_counter(pmu_type, event, *first_cpu_socket)?;
            let file = unsafe { File::from_raw_fd(fd) };
            let domain = PerfRaplDomain::new(domain_type, socket.socket_id, scale, file);

            debug!(
                "Opened domain {:?} on socket {}, fd: {}",
                domain_type, socket.socket_id, fd
            );

            domains.push(domain);
        }
    }

    info!("Opened {} RAPL domains", domains.len());
    Ok(domains)
}

/// Open a perf counter for a given event on a CPU.
///
/// # Errors
///
/// Returns `RaplError` if perf counter cannot be opened.
fn open_domain_counter(pmu_type: u32, event: u64, cpu: u32) -> Result<RawFd> {
    let mut attr: perf_event_attr = unsafe { std::mem::zeroed() };
    attr.type_ = pmu_type;
    attr.size = std::mem::size_of::<perf_event_attr>() as u32;
    attr.config = event;

    let fd = unsafe {
        perf_event_open(
            &mut attr as *mut perf_event_attr,
            -1,
            cpu as i32,
            -1,
            PERF_FLAG_FD_CLOEXEC,
        )
    };

    if fd < 0 {
        let err = std::io::Error::last_os_error();
        match err.kind() {
            std::io::ErrorKind::PermissionDenied => {
                return Err(RaplError::FailToOpenDomainCounter(
                    "try with root privileges or decrease perf_events paranoid level.".to_string(),
                ));
            }
            _ => {
                error!(
                    "Failed to open perf counter (cpu={}, event={}): {}",
                    cpu, event, err
                );
                return Err(err.into());
            }
        }
    }

    Ok(fd)
}

/// Read the scaling factor for a RAPL domain from sysfs.
fn read_domain_scale(domain_name: &str) -> Result<f64> {
    let path = format!("{}/events/{}.scale", PERF_RAPL_PATH, domain_name);
    fs::read_to_string(path)?
        .trim()
        .parse::<f64>()
        .map_err(Into::into)
}

/// Read the event number of a RAPL domain from sysfs.
///
/// # Errors
///
/// Returns `RaplError::InvalidEventFormat` if the event string cannot be parsed.
fn read_domain_event_number(domain: &str) -> Result<u64> {
    let type_path = format!("{}/events/{}", PERF_RAPL_PATH, domain);
    let content = fs::read_to_string(&type_path)?.trim().to_string();

    let hex_str = content
        .strip_prefix("event=0x")
        .or_else(|| content.strip_prefix("event="))
        .ok_or_else(|| RaplError::InvalidEventFormat(domain.to_string()))?;

    u64::from_str_radix(hex_str, 16).map_err(|_| RaplError::InvalidEventFormat(domain.to_string()))
}
