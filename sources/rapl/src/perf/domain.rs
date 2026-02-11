use std::{collections::HashSet, io::ErrorKind};

use log::debug;
use perf_event::{Builder, Group, ReadFormat, events::Software};

use crate::{
    Result,
    domain_type::RaplDomainType,
    error::{PerfParanoidError, RaplError},
    perf::{
        event::{RaplEvent, open_counters},
        socket::{Socket, SocketInfo, discover_socket_topology},
    },
};

/// Represents a RAPL domain with its perf_event counter.
///
/// Each domain corresponds to a measurable component like `PACKAGE-0`, `DRAM-0`, etc.
#[derive(Debug)]
pub struct PerfRaplDomain {
    /// Domain type (package, dram, core, etc.).
    pub domain_type: RaplDomainType,

    pub event_counter: RaplEvent,
}

impl PerfRaplDomain {
    /// Create a new `PerfRaplDomain`.
    pub fn new(domain_type: RaplDomainType, event_counter: RaplEvent) -> Self {
        Self {
            domain_type,
            event_counter,
        }
    }

    /// Apply the domain-specific scaling factor to convert raw value to joules.
    pub fn compute_scale(&self, value: u64) -> f64 {
        value as f64 * self.event_counter.scale
    }

    /// Return the domain name including socket, e.g., `PACKAGE-0`.
    pub fn get_name(&self, socket: u32) -> String {
        self.domain_type.to_string_socket(socket)
    }
}

/// Attempts to create a perf `Group` for a given socket by trying each CPU
/// in the provided list until one succeeds.
///
/// # Arguments
///
/// * `socket_info` - The information of a socket (e.g. its id and associated cpus)
///
/// # Returns
///
/// * `Ok(Group)` - The first successfully built perf group.
/// * `Err(RaplError::FailToOpenDomainCounter)` - If no CPU can be used to build the group.
///
/// # Notes
///
/// * Some PMUs, such as RAPL energy counters, are only accessible via a
///   specific CPU on the socket (often the first CPU). This function ensures
///   portability by trying all CPUs in the socket.
pub fn build_group_for_socket(socket_info: &SocketInfo) -> Result<Group> {
    for cpu in &socket_info.cpus_id {
        debug!("Trying to build perf group on CPU {}", cpu);

        match Builder::new(Software::DUMMY)
            .read_format(ReadFormat::GROUP)
            .one_cpu(*cpu as usize)
            .any_pid()
            .exclude_kernel(false)
            .exclude_hv(false)
            .build_group()
        {
            Ok(group) => {
                debug!("Perf group successfully built on CPU {}", cpu);
                return Ok(group);
            }
            Err(err) => {
                debug!("Failed to build group on CPU {}: {:?}", cpu, err);
                match err.kind() {
                    ErrorKind::PermissionDenied => {
                        return Err(PerfParanoidError::IoError(err).into());
                    }
                    _ => continue,
                }
            }
        }
    }

    Err(RaplError::FailToOpenDomainCounter(format!(
        "unable to find a CPU associate to the {} socket's PMU",
        socket_info.socket_id
    )))
}

/// Discover the system's socket topology.
///
/// # Arguments
///
/// * `domains_to_discover` - Optional filter for socket IDs.
///
/// # Returns
///
/// A vector of `SocketInfo` with socket IDs and associated CPUs.
///
/// # Errors
///
/// Returns `RaplError` if socket topology cannot be determined.
pub fn discover_domains(domains_to_discover: Option<&HashSet<u32>>) -> Result<Vec<SocketInfo>> {
    let socket_topology = discover_socket_topology(domains_to_discover)?;
    debug!("Socket topology: {:?}", socket_topology);
    Ok(socket_topology)
}

/// Discover available RAPL domains and open perf counters for each.
///
/// Convenience function that combines `discover_domains` and `open_counters`.
///
/// # Arguments
///
/// * `domains_to_discover` - Optional filter for socket IDs.
///
/// # Returns
///
/// A vector of initialized `Socket` instances.
///
/// # Errors
///
/// Returns `RaplError` if any perf counter cannot be opened or parsed.
pub fn discover_domains_and_open_counters(
    domains_to_discover: Option<&HashSet<u32>>,
) -> Result<Vec<Socket>> {
    let socket_topology = discover_domains(domains_to_discover)?;
    open_counters(socket_topology)
}
