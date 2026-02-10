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
  DRAM-0               | µJ
  CORE-0               | µJ
  PACKAGE-0            | µJ
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
│ RAPL (Powercap)                                │
└────────────────────────────────────────────────┘
  Name                 | Unit 
 ────────────────────────────────────────────────
  PACKAGE-0            | µJ
  CORE-0               | µJ
  DRAM-0               | µJ
```