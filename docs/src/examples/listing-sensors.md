# Sensors Listing Example

This example shows how to list all available sensors using Joule Profiler.

## Minimal Example

```bash
./target/debug/joule-profiler list-sensors
```

> [!NOTE]
> You should be able to list sensors without root privileges.

And it shows:
```
╔════════════════════════════════════════════════╗
║  Available Sensors                             ║
╚════════════════════════════════════════════════╝

┌────────────────────────────────────────────────┐
│ RAPL (perf_event)                              │
└────────────────────────────────────────────────┘
  Name                 | Unit 
 ────────────────────────────────────────────────
  PACKAGE-0            | µJ
  CORE-0               | µJ
  UNCORE-0             | µJ
  PSYS                 | µJ
```

## With GPU support

If you want to list also your GPU devices, use the `--gpu` CLI flag:

```bash
./target/debug/joule-profiler --gpu list-sensors
```

```
╔════════════════════════════════════════════════╗
║  Available Sensors                             ║
╚════════════════════════════════════════════════╝

┌────────────────────────────────────────────────┐
│ NVML                                           │
└────────────────────────────────────────────────┘
  Name                 | Unit 
 ────────────────────────────────────────────────
  GPU-0                | mJ

┌────────────────────────────────────────────────┐
│ RAPL (perf_event)                              │
└────────────────────────────────────────────────┘
  Name                 | Unit 
 ────────────────────────────────────────────────
  PACKAGE-0            | µJ
  CORE-0               | µJ
  UNCORE-0             | µJ
  PSYS                 | µJ
```

## With perf_event support

If you want to list also your GPU devices, use the `--perf` CLI flag:

```bash
./target/debug/joule-profiler --perf list-sensors
```

```
╔════════════════════════════════════════════════╗
║  Available Sensors                             ║
╚════════════════════════════════════════════════╝

┌────────────────────────────────────────────────┐
│ RAPL (perf_event)                              │
└────────────────────────────────────────────────┘
  Name                 | Unit 
 ────────────────────────────────────────────────
  PACKAGE-0            | µJ
  CORE-0               | µJ
  UNCORE-0             | µJ
  PSYS                 | µJ

┌────────────────────────────────────────────────┐
│ perf_event                                     │
└────────────────────────────────────────────────┘
  Name                 | Unit 
 ────────────────────────────────────────────────
  CPU_CYCLES           | count
  INSTRUCTIONS         | count
  CACHE_MISSES         | count
  BRANCH_MISSES        | count
```