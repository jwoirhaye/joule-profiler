# Quick Start

Getting started with **Joule Profiler** is easy and straightforward.
First, ensure you installed the profiler from [here](installation/quick-install.md).

## Basic Measurement

The simplest way to measure energy consumption is to run your program with the profiler:

```bash
# Simple measurement (terminal output)
sudo joule-profiler phases -- ./my-program arg1 arg2
```

This will execute your program and display energy consumption metrics in the terminal once it completes.

## Custom Output Format

By default, the output is shown in the terminal with a human-readable format. For further analysis or integration with other tools, you can export results in different formats:

```bash
# Export results as JSON for programmatic analysis
sudo joule-profiler --json phases -- ./my-program

# Export results as CSV for spreadsheet analysis
sudo joule-profiler --csv phases -- ./my-program
```

These formats make it easy to process results with scripts or import them into visualization tools.

## With Logging

To understand what the profiler is doing or troubleshoot issues, you can enable logging at different verbosity levels:

```bash
# Info-level logs (-v) for general information
sudo joule-profiler -v phases -- ./my-program

# Debug logs (-vv) for detailed diagnostic information
sudo joule-profiler -vv phases -- ./benchmark

# Trace logs (-vvv) for maximum verbosity
sudo joule-profiler -vvv phases -- ./my-program
```

Higher verbosity levels are useful for debugging but may produce large amounts of output and may be used only for debugging purposes, due to the introduced overhead of the I/O operations.

## Additional Options

### Specify an Output File

You can choose where to export your results:

```bash
# Save results to a file (format determined by extension or --json/--csv flags)
sudo joule-profiler -o results.json --json phases -- ./my-program
```

This is particularly useful when running multiple benchmarks.

### GPU Profiling

If your system has an Nvidia GPU and you want to measure GPU energy consumption alongside CPU:

```bash
# Include GPU metrics in the measurement
sudo joule-profiler --gpu phases -- ./my-program
```

### Performance Counters

**Joule Profiler** supports [perf_event](sources/perf_event/introduction.md) performance counters on Linux systems, you can activate this feature with the `--perf` flag.

```bash
sudo joule-profiler --perf phases -- ./my-program
```

GPU profiling requires the NVML library (part of NVIDIA driver installation).

### Choosing RAPL Backend

By default, **Joule Profiler** measures [**RAPL**](sources/rapl/introduction.md) counters using either [**perf_event**](sources/rapl/perf-event.md) (default) or [**Powercap**](sources/rapl/powercap.md) backend depending on your system.

You can explicitly choose which backend to use:

```bash
# Use Powercap backend (requires root)
sudo joule-profiler --rapl-backend=powercap phases -- ./my-program

# Use perf_event backend (lower overhead, may require kernel configuration)
sudo joule-profiler --rapl-backend=perf phases -- ./my-program
```

The choice of backend can affect measurement granularity and permission requirements. See the [RAPL](sources/rapl/introduction.md) documentation for details on each backend.

---

For more advanced usage including phase-based profiling, iterations, and custom metric sources, see the [examples folder](examples/overview.md).