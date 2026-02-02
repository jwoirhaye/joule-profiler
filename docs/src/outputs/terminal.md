# Terminal Output Format

When using Joule Profiler in the **terminal** (default output), the results are displayed in a **human-readable, structured format**.  
This section explains the different sections and the information shown.

## Command Summary

The first block always shows the command executed:

```
╔════════════════════════════════════════════════╗
║  Command                                       ║
╚════════════════════════════════════════════════╝
  python3 nbody.py 500000
```

- **Command** – Program and arguments executed.

## Phase Information

Phases mark sections of the program you want to measure. By default, the profiler includes a single phase: `START -> END`.

```
╔════════════════════════════════════════════════╗
║  Phase: START -> END                           ║
╚════════════════════════════════════════════════╝
  Duration            :       1878 ms
  Start token         :      START
  End token           :        END
```

- **Phase name** – Interval covered by the phase (can include custom tokens if the program outputs them).
- **Duration** – Time spent in this phase.
- **Start / End token** – Detected markers in the program output.

## Metrics per Phase

After the phase information, metrics from each source are displayed:

```
┌────────────────────────────────────────────────┐
│ powercap                                       │
└────────────────────────────────────────────────┘
  CORE-0              :   46068364 µJ
  DRAM-0              :    1287350 µJ
  PACKAGE-0           :   66901990 µJ
```

- **Source header** – Name of the metric source (`powercap`, `nvidia-nvml`, `perf`, etc.)
- **Sensors example** – Each sensor measured, showing:
  - `name` – e.g., `CORE-0`, `DRAM-0`, `PACKAGE-0`
  - `value` – Energy consumed
  - `unit` – `µJ` (microjoules)

> All sources are reported for each phase, allowing a complete view of program metrics.

## Minimal Example (Single Phase, Single Iteration)

```
╔════════════════════════════════════════════════╗
║  Command                                       ║
╚════════════════════════════════════════════════╝
  python3 nbody.py 500000

╔════════════════════════════════════════════════╗
║  Phase: START -> END                           ║
╚════════════════════════════════════════════════╝
  Duration            :       1878 ms
  Start token         :      START
  End token           :        END

┌────────────────────────────────────────────────┐
│ powercap                                       │
└────────────────────────────────────────────────┘
  CORE-0              :   46068364 µJ
  DRAM-0              :    1287350 µJ
  PACKAGE-0           :   66901990 µJ
```

## Multiple Iterations

If multiple iterations are run, each iteration is displayed as a separate block:

```
╔════════════════════════════════════════════════╗
║  Iteration 1 / 2                               ║
╚════════════════════════════════════════════════╝
  Duration            :       1885 ms
  Exit code           :          0
```

- **Iteration X / Y** – Indicates the iteration number and total iterations.
- **Duration / Exit code** – Time and return code for this iteration.
- Phases and metrics are repeated for each iteration.

> Multiple iterations improve measurement accuracy and reduce variance.

## Listing Sensors

Here is an example of sensors listing in terminal format:

```
╔════════════════════════════════════════════════╗
║  Available Sensors                             ║
╚════════════════════════════════════════════════╝
  Name                 | Unit  | Source         
 ────────────────────────────────────────────────
  PSYS-1               | µJ    | powercap       
  PACKAGE-0            | µJ    | powercap       
  CORE-0               | µJ    | powercap       
  UNCORE-0             | µJ    | powercap       
```