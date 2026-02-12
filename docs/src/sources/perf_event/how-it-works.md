# How It Works

## Kernel Counter Management

The Linux kernel manages performance counters through the `perf_event` subsystem. When you open a perf_event, the kernel:

1. **Allocates a counter**: Reserves a hardware or software counter
2. **Tracks timing**: Records when the counter is enabled and when it's actually running
3. **Reads hardware**: Periodically reads the hardware Performance Monitoring Unit (PMU) registers
4. **Updates state**: Maintains counter values across context switches and timer interrupts

When you read a counter, the kernel returns:
- **value**: The raw counter value
- **time_enabled**: Total time the counter has been enabled
- **time_running**: Actual time the counter was running (may be less due to multiplexing)

The kernel updates these values at context switches, timer interrupts, and when explicitly read.

## Counter Multiplexing

Hardware PMUs have limited counters (typically 4-8 per CPU core). When more events are requested than available counters, the kernel multiplexes them:

- Events are organized into groups
- The kernel rotates groups every ~1ms
- Only one group runs at a time
- When a counter isn't running, the kernel tracks this in `time_running` vs `time_enabled`

> [!NOTE]
> - When counters are multiplexed, scaling is applied to estimate real values, which can introduce measurement error.

## Memory-Mapped Access

The standard `read()` system call requires a context switch (100-300ns overhead). The kernel offers an mmap interface for lower overhead:

- The kernel maps a metadata page into user space
- User space can read counter state directly from this shared memory via CPU instructions (e.g., `rdpmc`) without any system call (10-50ns)

This is useful for high-frequency monitoring where system call overhead would be significant.

## References

- [perf_event_open(2) man page](https://man7.org/linux/man-pages/man2/perf_event_open.2.html)
- [Linux kernel perf documentation](https://www.kernel.org/doc/html/latest/admin-guide/perf-security.html)