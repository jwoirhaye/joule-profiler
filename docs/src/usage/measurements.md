# Measurements

## Overview

The profiler minimizes measurement overhead by performing only lightweight operations during program execution and deferring heavy computations until after completion.

## Key Principles

**Minimal Runtime Overhead**: During execution, we only:
- Read sources sensors
- Compute simple arithmetic deltas
- Store raw values in memory

**Deferred Computation**: We avoid during measurement:
- String formatting or allocations
- Data aggregation or statistics
- Heavy memory allocations and syscalls

## Measurement Process

### Initialization

To ensure accurate measurements from the first instruction:

1. The program is spawned as a child process
2. Immediately paused using `SIGSTOP` before execution begins
3. The process ID (PID) is shared with all metric sources via atomic storage and operations
4. Sources attach their instrumentation (e.g., perf_event counters scoped to the PID)
5. The program is resumed with `SIGCONT`

This eliminates race conditions and allow pid filtering for sources implementing it.


### Concurrent Measurement

Each metric source runs in its own asynchronous task. When a measurement is triggered:
- All sources read their counters in parallel
- Each source computes its delta independently
- Raw values are stored without transformation

This concurrent approach reduces total measurement time and ensures precise timing control; the measurements are made when we decide it.

### Raw Data Collection

Measurements occur at specific points:
- Initial baseline after program resume
- Phase transitions (detected via regex pattern in stdout)
- Final measurement after completion

At each point, sources store only raw numerical deltas without heavy computation on the data.

### Post-Measurement Processing

Only after the program completes do we perform expensive operations:
- Convert raw values into structured Metric objects with names, units, and metadata
- Aggregate results across all sources

These calculation do not have any impact on the results, because the measurements are already made and the profiled program has already exited.