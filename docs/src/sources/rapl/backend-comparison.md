# Backend Comparison

## Should you use Powercap of perf_event ?

Powercap is a framework for controlling and limiting power while perf_event is a tool for measuring performance counters.  
Thus, their design differs from each other, perf_event uses kernel mechanisms to minimize user–kernel transitions during data collection, reducing overhead for frequent measurements, while Powercap relies on a sysfs interface, where each read or write triggers a kernel entry, making it more suitable for infrequent operations or control tasks.

While they both use the same underlying technology (e.g., **MSRs for Intel RAPL**), they operate at different abstraction layers. perf_event provides a **measurement-oriented interface** optimized for profiling, whereas Powercap provides a **control-oriented interface** suitable for setting power limits or enforcing budgets.

| Scenario | Recommended Interface | Reason |
|----------|--------------------|--------|
| High-frequency, fine-grained energy measurement | perf_event | Minimal overhead introduced and less transition from user to kernel space |
| Moderate to low-frequency | perf_event or Powercap | Syscall overhead is acceptable, perf_event requires more configuration (perf_event_paranoid), while powercap is easy to use |

**Summary:** You should always prefer to use perf_event if it is configured on your system, but powercap is turnkey and easy to use.

## Why not use MSRs ?

We can access MSRs through the filesystem at `/dev/cpu/{core}/msr`, therefore, in principle we could read RAPL counters directly from the registers to minimize overhead.

In practice, direct MSR access from userspace does not necessarily provide better performance[^dissecting_software-based_measurement] than the powercap interface, which is already optimized for safe and efficient energy accounting. Reading MSRs from userspace requires a system call for each access, and repeated reads across multiple domains increase overhead. As a result, user-space MSR reads are generally slower than accessing the same counters through kernel-level drivers such as powercap, which can read and process the registers efficiently without repeated user to kernel context switches. In addition, raw MSR reads can exhibit greater variability at short time scales due to the absence of kernel-managed aggregation and coordinated sampling, whereas powercap provides more consistent and reproducible energy measurements by performing aggregation entirely within the kernel.

Moreover, using MSRs directly requires explicit management of overflow, unit scaling, and platform-specific behavior, reduces portability, and can introduce safety and consistency issues that higher-level kernel interfaces such as Linux's powercap framework handle automatically.