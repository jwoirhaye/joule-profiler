use std::collections::HashSet;
use std::mem::size_of_val;

use libc::{CPU_ISSET, cpu_set_t, sched_getaffinity};

use crate::profiler::types::Result;

pub fn get_process_sched_affinity(pid: i32) -> Result<HashSet<u16>> {
    let mut cpuset: cpu_set_t = unsafe { std::mem::zeroed() };

    let res =
        unsafe { sched_getaffinity(pid, size_of_val(&cpuset), &mut cpuset as *mut cpu_set_t) };

    if res != 0 {
        return Err(std::io::Error::last_os_error().into());
    }

    let mut cpus = HashSet::new();

    for cpu in 0..libc::CPU_SETSIZE as usize {
        let is_set = unsafe { CPU_ISSET(cpu, &cpuset) };
        if is_set {
            cpus.insert(cpu as u16);
        }
    }

    Ok(cpus)
}
