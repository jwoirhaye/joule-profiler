# Terminal Output Format

When using Joule Profiler in the **terminal** (default output), the results are displayed in a **human-readable, structured format**.  
This section explains the different sections and the information shown.

## Header

The header includes theses fields by default:
- **Phase name** - Interval covered by the phase (can include custom tokens if the program outputs them).
- **Duration** - Time spent in this phase.
- **Start / End token** - Detected markers in the program output.

Phases mark sections of the program you want to measure. By default, the profiler includes a single phase: `START -> END`.

```
╔════════════════════════════════════════════════╗
║  Phase: START -> END                           ║
╚════════════════════════════════════════════════╝
  Duration            :       1878 ms
  Start token         :      START
  End token           :        END
```

After the phase information, metrics from each source are displayed:

```
┌────────────────────────────────────────────────┐
│ RAPL (perf_event)                              │
└────────────────────────────────────────────────┘
  CORE-0              :   46068364 µJ
  DRAM-0              :    1287350 µJ
  PACKAGE-0           :   66901990 µJ
```

Metrics are grouped by metric source (e.g., `powercap`, `nvidia-nvml`, `perf`, etc.)

> [!NOTE]
> All metrics are reported for each phase.

## Profile Example

The command:
```
joule-profiler profile -- python3 nbody.py 500000
```

Shows:
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
│ RAPL (perf_event)                              │
└────────────────────────────────────────────────┘
  CORE-0              :   46068364 µJ
  DRAM-0              :    1287350 µJ
  PACKAGE-0           :   66901990 µJ
```

## Listing Sensors

Here is an example of sensors listing in terminal format:

```
joule-profiler list-sensors
```

Outputs:
```
╔════════════════════════════════════════════════╗
║  Available Sensors                             ║
╚════════════════════════════════════════════════╝
  Name                 | Unit  | Source         
 ────────────────────────────────────────────────
  PSYS-1               | µJ    | RAPL (perf_event)       
  PACKAGE-0            | µJ    | RAPL (perf_event)
  CORE-0               | µJ    | RAPL (perf_event)
  UNCORE-0             | µJ    | RAPL (perf_event)
```