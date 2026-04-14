# Quick Start

Getting started with **Joule Profiler** is easy and straightforward.

First, install the latest version:

```bash
curl -fsSL https://raw.githubusercontent.com/jwoirhaye/joule-profiler/main/install.sh | bash
```

## Basic Measurement

The simplest way to measure energy consumption is to run your program with the profiler:

```bash
joule-profiler profile -- ./my-program arg1 arg2
```

This will execute your program and display energy consumption metrics in the terminal once it completes:

```
╔════════════════════════════════════════════════╗
║  Command                                       ║
╚════════════════════════════════════════════════╝
  python3 main.py
 ────────────────────────────────────────────────
  Duration            :       3013 ms
  Exit code           :          0

╔════════════════════════════════════════════════╗
║  Phase: START -> END                           ║
╚════════════════════════════════════════════════╝
  Duration            :       3013 ms
  Start token         :      START
  End token           :        END

┌────────────────────────────────────────────────┐
│ RAPL (perf_event)                              │
└────────────────────────────────────────────────┘
  CORE-0              :    7299194 µJ
  PACKAGE-0           :   15104492 µJ
  PSYS                :   35101074 µJ
  UNCORE-0            :     355224 µJ
```

## Phases

Put some prints separating your program's parts:

```py
...

print("__SETUP_PHASE__", flush=True)

setup()

print("__WORKLOAD_PHASE__", flush=True)

workload()

print("__CLEANUP_PHASE__", flush=True)

cleanup()

...
```

Don't forget to flush the standard output after each print, see [troubleshooting](troubleshooting/overview.md).

Now, lauch Joule Profiler:

```bash
joule-profiler profile -- ./my-program arg1 arg2
```

You will get something like:

```
__SETUP__
__WORKLOAD__
__CLEANUP__
╔════════════════════════════════════════════════╗
║  Command                                       ║
╚════════════════════════════════════════════════╝
  python3 main.py
 ────────────────────────────────────────────────
  Duration            :       3013 ms
  Exit code           :          0

╔════════════════════════════════════════════════╗
║  Phase: START -> __SETUP__                     ║
╚════════════════════════════════════════════════╝
  Duration            :         10 ms
  Start token         :      START
  End token           : __SETUP__ (line 0)

┌────────────────────────────────────────────────┐
│ RAPL (perf_event)                              │
└────────────────────────────────────────────────┘
  CORE-0              :     184997 µJ
  PACKAGE-0           :     226806 µJ
  PSYS                :     387084 µJ
  UNCORE-0            :       4150 µJ

╔════════════════════════════════════════════════╗
║  Phase: __SETUP__ -> __WORKLOAD__              ║
╚════════════════════════════════════════════════╝
  Duration            :       1000 ms
  Start token         : __SETUP__ (line 0)
  End token           : __WORKLOAD__ (line 1)

┌────────────────────────────────────────────────┐
│ RAPL (perf_event)                              │
└────────────────────────────────────────────────┘
  CORE-0              :    1856750 µJ
  PACKAGE-0           :    3949645 µJ
  PSYS                :    8571228 µJ
  UNCORE-0            :      85571 µJ

╔════════════════════════════════════════════════╗
║  Phase: __WORKLOAD__ -> __CLEANUP__            ║
╚════════════════════════════════════════════════╝
  Duration            :       1000 ms
  Start token         : __WORKLOAD__ (line 1)
  End token           : __CLEANUP__ (line 2)

┌────────────────────────────────────────────────┐
│ RAPL (perf_event)                              │
└────────────────────────────────────────────────┘
  CORE-0              :    1398742 µJ
  PACKAGE-0           :    3311218 µJ
  PSYS                :    7377380 µJ
  UNCORE-0            :      58105 µJ

╔════════════════════════════════════════════════╗
║  Phase: __CLEANUP__ -> END                     ║
╚════════════════════════════════════════════════╝
  Duration            :       1003 ms
  Start token         : __CLEANUP__ (line 2)
  End token           :        END

┌────────────────────────────────────────────────┐
│ RAPL (perf_event)                              │
└────────────────────────────────────────────────┘
  CORE-0              :    1468383 µJ
  PACKAGE-0           :    3395080 µJ
  PSYS                :    7314697 µJ
  UNCORE-0            :     137817 µJ
```

## Custom Output Format

By default, the output is shown in the terminal with a human-readable format. For further analysis or integration with other tools, you can export results in different formats:

```bash
# Export results as JSON for programmatic analysis
joule-profiler --json profile -- ./my-program

# Export results as CSV for spreadsheet analysis
joule-profiler --csv profile -- ./my-program
```

An output file will be generated at the end of the profiling.
These formats make it easy to process results with scripts or import them into visualization tools.

## Additional Options

### Specify an Output File

You can choose where to export your results:

```bash
# Save results to a file (format determined by extension or --json/--csv flags)
joule-profiler -o results.json --json profile -- ./my-program
```

This is particularly useful when running multiple benchmarks.

### GPU Profiling

If your system has an Nvidia GPU and you want to measure GPU energy consumption alongside CPU:

```bash
# Include GPU metrics in the measurement
joule-profiler --gpu profile -- ./my-program
```

### Performance Counters

**Joule Profiler** supports [perf_event](sources/perf_event/introduction.md) performance counters on Linux systems, you can activate this feature with the `--perf` flag.

```bash
joule-profiler --perf profile -- ./my-program
```

GPU profiling requires the NVML library (part of NVIDIA driver installation).

### Choosing RAPL Backend

By default, **Joule Profiler** measures [**RAPL**](sources/rapl/introduction.md) counters using either [**perf_event**](sources/rapl/perf-event.md) (default) or [**Powercap**](sources/rapl/powercap.md) backend depending on your system.

You can explicitly choose which backend to use:

```bash
# Use Powercap backend (requires root)
sudo joule-profiler --rapl-backend=powercap profile -- ./my-program

# Use perf_event backend (lower overhead, may require kernel configuration)
joule-profiler --rapl-backend=perf profile -- ./my-program
```

The choice of backend can affect measurement granularity and permission requirements. See the [RAPL](sources/rapl/introduction.md) documentation for details on each backend.

---

For more advanced usage including phase-based profiling, iterations, and custom metric sources, see the [examples folder](examples/overview.md).