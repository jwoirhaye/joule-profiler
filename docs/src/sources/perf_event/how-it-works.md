# How It Works

## Kernel Counter Management

The Linux kernel manages performance counters through the `perf_event` subsystem. When a perf_event counter is opened, the Linux kernel allocates a counter and periodically reads the hardware Performance Monitoring Unit (PMU) registers. It maintains counter values across context switches and timer interrupts.

## Counter Multiplexing

Hardware PMUs have limited counters (typically 4-8 per CPU core). When more events are requested than available counters, the kernel multiplexes them.
The kernel multiplexing organize counters into groups and rotates them around every millisecond, meaning that only one group of counters can run at a time.
To solve this issue, perf_event keeps track of when the timer is enabled and when it's actually running, allowing to scale them when they are multiplexed. However, counter scaling can introduce an error because it is an estimation of the global value out of different local ones, thus weakening the detection of small variabilities.

## Memory-Mapped Access

The standard `read()` system call requires a context switch (100-300ns overhead). The kernel offers an mmap interface for lower overhead. It maps a kernel memory page into user space, thus allowing to read counter state directly from this share memory via CPU instructions (e.g., `rdpmc`) without any system call taking 10 to 50 nanoseconds.

This is useful for high-frequency monitoring where system call overhead would be significant.

For now, **Joule Profiler** is using the `read` syscall, but we may implement an mmap version to reduce the overhead of the syscalls.

## References

- [perf_event_open(2) man page](https://man7.org/linux/man-pages/man2/perf_event_open.2.html)
- [Linux kernel perf documentation](https://www.kernel.org/doc/html/latest/admin-guide/perf-security.html)