# Measurements

## Overview

The profiler minimizes the measurements overhead by performing only the required operations during program execution and deferring heavy computations like data transformation until the end of the profiling.

**Minimal Runtime Overhead** During execution, we only:
- Read sources sensors
- Store raw values in memory

**Deferred Computation** We avoid during measurement:
- String formatting or allocations
- Data aggregation or transformation
- Heavy memory allocations and syscalls

## Measurement Process

### Initialization

To ensure accurate measurements from the first instruction:

1. The program is spawned as a child process
2. Immediately paused using `SIGSTOP` before execution begins
3. The process ID (PID) is shared with all metric sources via atomic storage and operations
4. Sources attach their instrumentation (e.g. [perf_event](../sources/perf_event/introduction.md) counters scoped to the PID)
5. The program is resumed with `SIGCONT`

This allows pid filtering for sources implementing it, while minimizing the introduced overhead by stopping the program during the initialization of sources requiring per-process measurement.

### Concurrent Measurement

Each metric source runs in its own asynchronous task. When a measurement is triggered:
- All sources read their counters in parallel
- Raw values from each source are stored separately without transformation

This concurrent approach allows to make the measurement of all sources at the same time and ensures precise timing control.

### Raw Data Collection

Measurements occur at specific points:
- Initial baseline after program resume
- Phase transitions (detected via regex pattern in stdout)
- Final measurement after completion

### Post-Measurement Processing

We perform expensive operations only after the program completion:
- Convert raw values into structured metric objects
- Aggregate results across all sources

These calculation do not have any impact on the results, because the measurements are already made and the profiled program has already exited.