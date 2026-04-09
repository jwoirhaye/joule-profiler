# perf_event_paranoid

## Overview

`perf_event_paranoid` is a kernel tunable that controls access permissions to performance monitoring events. It determines what level of privilege is required to use various `perf_event` features.

This setting is crucial for security, as unrestricted access to performance counters can potentially leak sensitive information about system behavior and other processes.

## Permission Levels

The `/proc/sys/kernel/perf_event_paranoid` file accepts integer values that define access restrictions:

| Level | Description | Access restrictions |
|-------|-------------|----------------|
| **-1** | No restrictions | All users can access all events, including kernel profiling and CPU-specific events. Not recommended. |
| **0** | Relaxed (default on some systems) | Unprivileged users can perform per-process profiling but cannot profile kernel space or other users' processes. |
| **1** | Moderate | Unprivileged users can only access CPU events (cycles, instructions). No access to kernel profiling or tracepoints. |
| **2** | Restricted (common default) | Unprivileged users cannot use `perf_event_open` at all. Only `CAP_PERFMON` or `CAP_SYS_ADMIN` capabilities allowed. |
| **3** | Fully restricted | Denies all access to perf events, even for privileged processes (rarely used). |
| **4** | Maximum restriction | Complete lockdown of perf subsystem. |

> [!NOTE]
> The default value varies by distribution but is set to 2 by default on most distro.

## Checking Current Setting

```bash
# View current paranoid level
cat /proc/sys/kernel/perf_event_paranoid
```

## Configuring perf_event_paranoid

```bash
# Set to level 1 (moderate restrictions)
sudo sysctl kernel.perf_event_paranoid=1

# Or directly write to proc
echo 1 | sudo tee /proc/sys/kernel/perf_event_paranoid
```

## Security Implications

Performance counters can expose sensitive information about other processes. An attacker could do a side-channel attack and measure execution time to infer cryptographic keys or other secrets, he could also observe cache behavior to extract data.

### Safe Practices

1. **Use capabilities instead of paranoid level**:
   ```bash
   # Grant CAP_PERFMON to specific binaries
   sudo setcap cap_perfmon=ep /path/to/joule-profiler
   ```

2. **Limit access to specific users**:
   ```bash
   # Add user to perf_users group (if your distro supports it)
   sudo usermod -aG perf_users $USER
   ```

3. **Run with sudo when needed**:
   ```bash
   sudo perf stat -e cycles ./my_program
   ```

## Troubleshooting

### "Permission denied" errors

```
Error: perf_event_paranoid level is 1, try setting it to 0 or launch Joule Profiler with root rights
```

**Solution**: Either lower `perf_event_paranoid` level, grant Joule Profiler `CAP_PERFMON` capability or launch it with root privileges (sudo).

> [!NOTE]
> To access RAPL counters using perf_event, you need to set perf_event_paranoid level to 0, or launch the profiler with root privileges.

## References

- [Linux kernel perf_event documentation](https://www.kernel.org/doc/html/latest/admin-guide/perf-security.html)
- [perf_event_open(2) man page](https://man7.org/linux/man-pages/man2/perf_event_open.2.html)