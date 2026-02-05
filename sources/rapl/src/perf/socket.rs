use std::{
    collections::{HashMap, HashSet},
    fs,
};

use log::{debug, trace};

use crate::Result;

const CPU_SYSFS_PATH: &str = "/sys/devices/system/cpu";
const CPU_TOPOLOGY_SOCKET_ID: &str = "/topology/physical_package_id";
const ONLINE_CPU_SYSFS_PATH: &str = "/sys/devices/system/cpu/online";

#[derive(Debug)]
pub struct SocketInfo {
    pub socket_id: u32,
    pub cpus_id: Vec<u32>,
}

pub fn discover_socket_topology(
    sockets_to_discover: Option<&HashSet<u32>>,
) -> Result<Vec<SocketInfo>> {
    let mut sockets: HashMap<u32, Vec<u32>> = HashMap::new();
    let online_cpus = read_online_cpus()?;
    trace!("Found {} cpu(s)", online_cpus.len());

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
        .map(|(socket_id, cpus_id)| {
            trace!("Found {:?} cpus for socket {}", cpus_id, socket_id);
            SocketInfo { socket_id, cpus_id }
        })
        .collect();

    debug!("Discovered {} socket(s)", socket_topology.len());

    Ok(socket_topology)
}

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
