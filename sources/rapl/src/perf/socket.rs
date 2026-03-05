use std::{
    collections::{HashMap, HashSet},
    fs,
};

use log::{debug, trace};
use perf_event::Group;

use crate::{Result, perf::domain::PerfRaplDomain};

/// Path to CPU sysfs directory.
const CPU_SYSFS_PATH: &str = "/sys/devices/system/cpu";

/// Relative path to the physical package (socket) ID in sysfs.
const CPU_TOPOLOGY_SOCKET_ID: &str = "/topology/physical_package_id";

/// Path to the online CPUs file in sysfs.
const ONLINE_CPU_SYSFS_PATH: &str = "/sys/devices/system/cpu/online";

const CPUMASK_SYSFS_PATH: &str = "/sys/devices/power/cpumask";

/// Represents a CPU socket and the list of CPUs it contains.
#[derive(Debug)]
pub struct SocketInfo {
    /// The ID of the socket.
    pub socket_id: u32,
    /// List of CPU IDs associated with this socket.
    pub cpus_id: Vec<u32>,
    /// cpumask of the socket.
    pub cpumask: Vec<u32>,
}

pub struct Socket {
    pub id: u32,
    pub group: Group,
    pub domains: Vec<PerfRaplDomain>,
}

/// Discover the CPU socket topology of the system.
///
/// # Arguments
///
/// * `sockets_to_discover` - Optional filter to discover only specific socket IDs.
///
/// # Returns
///
/// A `Vec<SocketInfo>` containing each discovered socket and its CPUs.
///
/// # Errors
///
/// Returns an error if reading sysfs files fails or parsing fails.
pub fn discover_socket_topology(
    sockets_to_discover: Option<&HashSet<u32>>,
) -> Result<Vec<SocketInfo>> {
    let mut sockets: HashMap<u32, Vec<u32>> = HashMap::new();
    let online_cpus = read_online_cpus()?;
    trace!("Found {} cpu(s)", online_cpus.len());

    let mut cpumask_per_socket = read_cpumask()?;

    for entry in fs::read_dir(CPU_SYSFS_PATH)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if !name.starts_with("cpu") {
            continue;
        }

        let cpu_id: u32 = if let Ok(id) = name.chars().skip(3).collect::<String>().parse() {
            id
        } else {
            continue;
        };

        if !online_cpus.contains(&cpu_id) {
            continue;
        }

        let pkg_path = format!("{}/{}{}", CPU_SYSFS_PATH, name, CPU_TOPOLOGY_SOCKET_ID);
        let socket_id: u32 = fs::read_to_string(pkg_path)?.trim().parse()?;

        if let Some(sockets) = sockets_to_discover
            && !sockets.contains(&socket_id)
        {
            continue;
        }

        sockets.entry(socket_id).or_default().push(cpu_id);
    }

    let socket_topology: Vec<SocketInfo> = sockets
        .into_iter()
        .map(|(socket_id, mut cpus_id)| {
            trace!("Found {:?} cpus for socket {}", cpus_id, socket_id);
            let cpumask = if let Some(cpumask) = cpumask_per_socket.remove(&socket_id) {
                trace!("cpumask for socket {}: {:?}", socket_id, cpumask);
                cpumask
            } else {
                trace!("Unable to find cpumask for socket {}", socket_id);
                vec![]
            };

            cpus_id.sort_unstable();
            SocketInfo {
                socket_id,
                cpus_id,
                cpumask,
            }
        })
        .collect();

    debug!("Discovered {} socket(s)", socket_topology.len());

    Ok(socket_topology)
}

/// Read the list of online CPUs from sysfs.
///
/// # Returns
///
/// A `HashSet<u32>` containing the IDs of all online CPUs.
///
/// # Errors
///
/// Returns an error if the sysfs file cannot be read or if parsing fails.
fn read_online_cpus() -> Result<HashSet<u32>> {
    let content = fs::read_to_string(ONLINE_CPU_SYSFS_PATH)?;
    let mut cpus = HashSet::new();

    for part in content.trim().split(',') {
        if let Some((start, end)) = part.split_once('-') {
            let start: u32 = start.parse()?;
            let end: u32 = end.parse()?;

            cpus.extend(start..=end);
        } else {
            let cpu: u32 = part.parse()?;
            cpus.insert(cpu);
        }
    }
    Ok(cpus)
}

fn read_cpumask() -> Result<HashMap<u32, Vec<u32>>> {
    let content = fs::read_to_string(CPUMASK_SYSFS_PATH)?;
    let mut cpus_per_socket = HashMap::new();

    for (i, part) in content.trim().split(',').enumerate() {
        let cpu_mask = if let Some((start, end)) = part.split_once('-') {
            let start: u32 = start.parse()?;
            let end: u32 = end.parse()?;

            (start..end).collect()
        } else {
            let cpu: u32 = part.parse()?;
            vec![cpu]
        };
        cpus_per_socket.insert(i as u32, cpu_mask);
    }
    Ok(cpus_per_socket)
}
