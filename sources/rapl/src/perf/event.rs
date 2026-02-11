use log::{debug, info, warn};
use perf_event::{Builder, Counter, Group, events::Dynamic};

use crate::{
    RaplError, Result,
    domain_type::RaplDomainType,
    perf::{
        domain::{PerfRaplDomain, build_group_for_socket},
        socket::{Socket, SocketInfo},
    },
};

#[derive(Debug)]
pub struct RaplEvent {
    pub counter: Counter,
    pub scale: f64,
}

impl RaplEvent {
    /// Attempts to create a RAPL event counter for a given domain on a socket,
    /// by trying each CPU in the socket until one succeeds.
    ///
    /// # Arguments
    ///
    /// * `domain_type` - Type of RAPL domain (package, dram, etc.)
    /// * `socket_info` - Socket information, including its associated CPUs
    /// * `group` - Mutable reference to the perf `Group` to attach the counter
    ///
    /// # Returns
    ///
    /// * `Ok(RaplEvent)` if a counter is successfully created
    /// * `Err(RaplError::FailToOpenDomainCounter)` if no CPU allows building the counter
    ///
    /// # Notes
    ///
    /// Some PMUs, like RAPL counters, may only be accessible through specific CPUs
    /// in the socket. This function ensures portability by trying all CPUs in the socket.
    pub fn new(
        domain_type: RaplDomainType,
        socket_info: &SocketInfo,
        group: &mut Group,
    ) -> Result<Self> {
        let mut builder = Dynamic::builder("power")?;
        if let Err(err) = builder.event(domain_type.to_perf_event()) {
            return Err(match err.kind() {
                std::io::ErrorKind::NotFound => RaplError::DomainNotSupported(domain_type),
                _ => err.into(),
            });
        }

        let scale = builder.scale()?.ok_or(RaplError::RetrieveScaleError)?;

        for cpu in &socket_info.cpus_id {
            debug!(
                "Trying to build RAPL event {:?} on CPU {}",
                domain_type, cpu
            );

            let counter_result = Builder::new(
                builder
                    .build()
                    .map_err(|err| RaplError::FailToOpenDomainCounter(err.to_string()))?,
            )
            .one_cpu(*cpu as usize)
            .any_pid()
            .exclude_hv(false)
            .exclude_kernel(false)
            .build_with_group(&mut *group);

            match counter_result {
                Ok(counter) => {
                    debug!(
                        "RAPL event {:?} successfully built on CPU {}",
                        domain_type, cpu
                    );
                    return Ok(Self { scale, counter });
                }
                Err(err) => {
                    debug!(
                        "Failed to build RAPL event {:?} on CPU {}: {:?}",
                        domain_type, cpu, err
                    );
                    continue;
                }
            }
        }

        Err(RaplError::FailToOpenDomainCounter(format!(
            "unable to find a CPU associated with the {} socket's PMU for domain {:?}",
            socket_info.socket_id, domain_type
        )))
    }
}

/// Open perf counters for the discovered RAPL domains.
///
/// # Arguments
///
/// * `socket_topology` - Vector of socket information from `discover_domains`.
///
/// # Returns
///
/// A vector of initialized `Socket` instances with their perf counters.
///
/// # Errors
///
/// Returns `RaplError` if any perf counter cannot be opened.
pub fn open_counters(socket_topology: Vec<SocketInfo>) -> Result<Vec<Socket>> {
    let mut sockets = Vec::new();

    for socket_info in socket_topology {
        debug!(
            "Processing socket {} (cpus={:?})",
            socket_info.socket_id, socket_info.cpus_id
        );

        if socket_info.cpus_id.is_empty() {
            warn!("Socket {} has no CPUs, skipping it", socket_info.socket_id);
            continue;
        }

        let mut group = build_group_for_socket(&socket_info)?;
        let domains = open_counters_for_socket(&socket_info, &mut group)?;

        info!(
            "Opened {} RAPL domains for socket {}",
            domains.len(),
            socket_info.socket_id
        );

        let socket = Socket {
            domains,
            group,
            id: socket_info.socket_id,
        };
        sockets.push(socket);
    }

    Ok(sockets)
}

static PER_SOCKET_DOMAIN_TYPES: [RaplDomainType; 4] = [
    RaplDomainType::Package,
    RaplDomainType::Core,
    RaplDomainType::Uncore,
    RaplDomainType::Dram,
];

/// Opens available RAPL counters for a CPU socket.
///
/// Adds counters for standard domains (Package, Core, Uncore, Dram) and
/// adds Psys only for socket 0.
///
/// # Arguments
/// * `socket_info` - Info about the target CPU socket.
/// * `group` - Group to which counters will be added.
///
/// # Returns
/// A vector of initialized `PerfRaplDomain` counters, or an error.
pub fn open_counters_for_socket(
    socket_info: &SocketInfo,
    group: &mut Group,
) -> Result<Vec<PerfRaplDomain>> {
    let mut domains = Vec::new();

    for domain_type in PER_SOCKET_DOMAIN_TYPES {
        add_counter_to_domain_if_supported(domain_type, socket_info, group, &mut domains)?;
    }

    if socket_info.socket_id == 0
        && let Some(psys_domain) = open_psys_counter(group)?
    {
        domains.push(psys_domain);
    }

    Ok(domains)
}

/// Attempts to open the PSYS (platform) domain counter.
///
/// PSYS is a platform-wide domain that measures total system power,
/// not tied to any specific socket. It should only be opened once.
///
/// # Arguments
/// * `group` - Group to attach the counter to (can use any CPU's group)
///
/// # Returns
/// * `Ok(Some(PerfRaplDomain))` if PSYS is available
/// * `Ok(None)` if PSYS is not supported on this system
/// * `Err(RaplError)` for other errors
pub fn open_psys_counter(group: &mut Group) -> Result<Option<PerfRaplDomain>> {
    let psys_socket_info = SocketInfo {
        socket_id: 0,
        cpus_id: vec![0],
    };

    match RaplEvent::new(RaplDomainType::Psys, &psys_socket_info, group) {
        Ok(counter) => {
            let domain = PerfRaplDomain::new(RaplDomainType::Psys, counter);
            Ok(Some(domain))
        }
        Err(RaplError::DomainNotSupported(_)) => {
            debug!("PSYS domain not supported on this system");
            Ok(None)
        }
        Err(err) => Err(err),
    }
}

pub fn add_counter_to_domain_if_supported(
    domain_type: RaplDomainType,
    socket_info: &SocketInfo,
    group: &mut Group,
    domains: &mut Vec<PerfRaplDomain>,
) -> Result<()> {
    let counter = match RaplEvent::new(domain_type, socket_info, group) {
        Ok(counter) => counter,
        Err(err) => match err {
            RaplError::DomainNotSupported(_) => return Ok(()),
            _ => {
                return Err(err);
            }
        },
    };
    let domain = PerfRaplDomain::new(domain_type, counter);
    domains.push(domain);
    Ok(())
}
