use std::{
    collections::HashSet,
    fs::{self, File},
    io::Read,
    os::fd::{AsRawFd, FromRawFd, RawFd},
};

use log::{debug, trace};
use perf_event_open_sys::{bindings::perf_event_attr, perf_event_open};

use crate::{
    Result,
    domain_type::RaplDomainType,
    error::RaplError,
    perf::{PERF_RAPL_PATH, socket::discover_socket_topology},
};

const PERF_FLAG_FD_CLOEXEC: u64 = 8;

#[derive(Debug)]
pub struct PerfRaplDomain {
    pub domain_type: RaplDomainType,
    pub socket: u32,
    pub scale: f64,
    pub fd: File,
}

impl PerfRaplDomain {
    pub fn new(domain_type: RaplDomainType, socket: u32, scale: f64, fd: File) -> Self {
        Self {
            domain_type,
            socket,
            scale,
            fd,
        }
    }

    /// Read the raw fd perf counters
    pub fn read_counter(&mut self) -> Result<u64> {
        let mut buf = [0u8; 8];
        self.fd.read_exact(&mut buf)?;
        Ok(u64::from_ne_bytes(buf))
    }

    pub fn compute_scale(&self, value: u64) -> f64 {
        value as f64 * self.scale
    }

    pub fn enable_counter(&self) {
        unsafe { perf_event_open_sys::ioctls::ENABLE(self.as_raw_fd(), 0) };
    }

    pub fn reset_counter(&self) {
        unsafe { perf_event_open_sys::ioctls::RESET(self.as_raw_fd(), 0) };
    }

    pub fn get_name(&self) -> String {
        self.domain_type.to_string_socket(self.socket)
    }
}

impl AsRawFd for PerfRaplDomain {
    fn as_raw_fd(&self) -> RawFd {
        self.fd.as_raw_fd()
    }
}

pub fn discover_domains_and_open_counters(
    pmu_type: u32,
    rapl_path: &str,
    domains_to_discover: Option<&HashSet<u32>>,
) -> Result<Vec<PerfRaplDomain>> {
    let mut domains = Vec::new();
    let domains_dir_path = format!("{}/events", rapl_path);
    let socket_topology = discover_socket_topology(domains_to_discover)?;

    for socket in &socket_topology {
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

            let event = read_domain_event_number(&file_name)?;
            let scale = read_domain_scale(&file_name)?;
            let domain_type = file_name.try_into()?;

            let first_cpu_socket = if let Some(first_socket_cpu) = socket.cpus_id.first() {
                first_socket_cpu
            } else {
                continue;
            };

            let fd = if let Ok(fd) = open_domain_counter(pmu_type, event, *first_cpu_socket) {
                debug!(
                    "Opened domain {} with cpu {} on socket {}",
                    domain_type, first_cpu_socket, socket.socket_id
                );
                fd
            } else {
                trace!(
                    "Unsupported domain {} for socket {}, skipped",
                    domain_type, socket.socket_id
                );
                continue;
            };

            let file = unsafe { File::from_raw_fd(fd) };
            let domain = PerfRaplDomain::new(domain_type, socket.socket_id, scale, file);
            domains.push(domain);
        }
    }

    Ok(domains)
}

fn open_domain_counter(pmu_type: u32, event: u64, cpu: u32) -> Result<RawFd> {
    let mut attr: perf_event_attr = unsafe { std::mem::zeroed() };
    attr.type_ = pmu_type;
    attr.size = std::mem::size_of::<perf_event_attr>() as u32;
    attr.config = event;
    attr.set_disabled(1);

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
        return Err(RaplError::FailToOpenDomainCounter(
            std::io::Error::last_os_error().to_string(),
        ));
    }

    Ok(fd)
}

fn read_domain_scale(domain_name: &str) -> Result<f64> {
    let path = format!("{}/events/{}.scale", PERF_RAPL_PATH, domain_name);
    fs::read_to_string(path)?
        .trim()
        .parse::<f64>()
        .map_err(Into::into)
}

fn read_domain_event_number(domain: &str) -> Result<u64> {
    let type_path = format!("{}/events/{}", PERF_RAPL_PATH, domain);
    let content = fs::read_to_string(&type_path)?.trim().to_string();

    let hex_str = content
        .strip_prefix("event=0x")
        .or_else(|| content.strip_prefix("event="))
        .ok_or_else(|| RaplError::InvalidEventFormat(domain.to_string()))?;

    u64::from_str_radix(hex_str, 16).map_err(|_| RaplError::InvalidEventFormat(domain.to_string()))
}
