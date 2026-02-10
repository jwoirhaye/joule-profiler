# Quick Start

Getting started with **Joule Profiler** is easy and straightforward.
First, ensure you installed the profiler from [here](installation/quick-install.md).

## Basic Measurement

Measure energy consumption of a program:

```bash
# Simple measurement (terminal output)
sudo joule-profiler phases -- ./my-program arg1 arg2
```

## Custom Output Format

By default, the output is shown in the terminal with a human-readable format.

You can use the `--json` or `--csv` CLI flags to change the output format:

```bash
# With JSON output
sudo joule-profiler --json phases -- ./my-program

# With CSV output
sudo joule-profiler --csv phases -- ./my-program
```

## With Logging

```bash
# Info-level logs (-v)
sudo joule-profiler -v phases -- ./my-program

# Debug logs (-vv) for detailed information
sudo joule-profiler -vv phases -- ./benchmark

# Trace logs (-vvv) for maximum verbosity
sudo joule-profiler -vvv phases -- ./my-program
```

## Additional Options

### Specify an Output File

```bash
# Specify an output file
sudo joule-profiler -o results phases -- ./my-program
```

### GPU Profiling

```bash
# With Nvidia GPU support
sudo joule-profiler --gpu phases -- ./my-program
```

### Choosing RAPL Backend

By default, only [**RAPL**](sources/rapl/introduction.md) counters are measured with [**perf_event**](sources/rapl/perf-event.md) or [**Powercap**](sources/rapl/powercap.md) backend depending on your system.

```bash
# With Powercap backend
sudo joule-profiler --rapl-backend=powercap phases -- ./my-program

# With perf_event backend
sudo joule-profiler --rapl-backend=perf phases -- ./my-program
```

For more advances usage, see the [examples folder](examples/overview.md).