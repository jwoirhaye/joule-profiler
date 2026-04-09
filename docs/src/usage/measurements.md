# Measurements

## Overview

Joule Profiler minimizes the measurements overhead by performances only the required operations during program execution. Heavy computations like string formatting or allocation, data aggregation or transformation, heavy memory allocations and syscalls are deferred after the end of the profiling. During the program' execution, it only reads sources sensors and store raw values in memory

## Measurement Process

### Initialization

To ensure accurate measurements from the first instruction:

1. The program is spawned as a child process
2. Immediately paused using `SIGSTOP` before execution begins
3. The process ID (PID) is shared with all metric sources via atomic storage and operations
4. Sources attach their instrumentation (e.g., [perf_event](../sources/perf_event/introduction.md) counters scoped to the PID)
5. The program is resumed with `SIGCONT`

This allows pid filtering for sources implementing it, while minimizing the introduced overhead by stopping the program during the initialization of sources requiring per-process measurement.

### Measurements

Each metric source runs in its own asynchronous task. When a measurement is triggered, every source reads their counters in parallel and stores raw values separately without data transformation.
This concurrent approach allows to make the measurement of all sources at the same time and ensures precise timing control.

Measurements occur at specific points, the first triggered measurement is the initial baseline before resuming the program, then measurements are made for each phase transitions detected via regex pattern in stdout. The Final measurement is made after the completion of the program.

### Post-Measurements Processing

We perform expensive operations only after the program completion. We convert raw sources values into structured metric objects and aggregate them across phases.
These calculation do not have any impact on the results because the measurements are already made and the profiled program has already exited.