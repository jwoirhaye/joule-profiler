# RAPL

## Overview

RAPL (Running Average Power Limit) is an Intel processor feature that allows real-time energy consumption measurements of CPU and memory subsystem.

This technology has been available on Intel processors since *Sandy Bridge* generation.

RAPL provides energy measurements at different scales, enabling you to measure energy consumption per component and understand more precisely how each part of the system contributes to the overall power usage.

It allows fine-grained energy profiling of CPU cores, memory subsystem, and uncore components.

## Architecture

RAPL interface exposes multiple power domains that allow measuring energy consumption of different parts of the processor and memory subsystem.
Domains metrics are accessible through model-specific registers (MSRs) on the host system, enabling user to monitor power usage in real time.

| Domain | Description |
|--------|------------|
| **Package** | Measures the total energy consumption of the entire CPU socket. This includes cores and uncore components. |
| **Core/PP0** | Represents the CPU cores only. Useful for profiling per-core energy consumption. |
| **Uncore/PP1** | Covers the energy consumption of last-level caches, memory controller, and may include the integrated GPU depending on the CPU generation. |
| **DRAM** | Measures the energy consumption of dynamic random access memory attached to the integrated memory controller if supported. |
| **PSYS** | Entire SoC energy consumption (available since *Skylake* generation) |

Some of the domains may not appear depending on the processor architecture. 

Architecture of RAPL[^dissecting_software-based_measurement]:

![RAPL architecture](./figures/rapl_architecture.png)


## Powercap

### Overview

Powercap[^powercap] is a Linux kernel framework that provides a generic and standardized interface for power capping and energy monitoring accross different hardware power domains.

Instead of accessing low-level hardware registers (e.g MSRs), Powercap safely exposes energy metrics via sysfs. 

The use of powercap instead of MSRs may seem disadvantageous and cause more overhead while measuring energy consumption, but there is actually no or an insignificant overhead introduced by the use of the powercap framework. Moreover, the abstraction provided by powercap increases the maintainibility.

### Sysfs file structure

The Powercap framework exports energy data through the `/sys/class/powercap/` directory. Each physical CPU socket or hardware component is represented as a **control type** (usually `intel-rapl`).

Within `intel-rapl`, the hierarchy is organized by **zones** and **subzones**, which correspond to the RAPL domains (Package, Core, DRAM, etc.).

The structure typically looks like this:

```text
/sys/class/powercap/
└── intel-rapl/
    ├── intel-rapl:0/                # Package 0 (CPU Socket 0)
    │   ├── name                     # Content: "package-0"
    │   ├── energy_uj                # Cumulative energy in microjoules
    │   ├── max_energy_range_uj      # Overflow value for the counter
    │   ├── intel-rapl:0:0/          # Subzone: Core (PP0)
    │   │   ├── name                 # Content: "core"
    │   │   └── energy_uj
    │   ├── intel-rapl:0:1/          # Subzone: DRAM
    │   │   ├── name                 # Content: "dram"
    │   │   └── energy_uj
    │   └── intel-rapl:0:2/          # Subzone: Uncore (PP1)
    │       ├── name                 # Content: "uncore"
    │       └── energy_uj
    └── intel-rapl:1/                # Package 1 (CPU Socket 1, if multi-socket)
```

To retrieve the domains measure energy consumption, the following files are accessed:

* **`name`**: The name of the corresponding domain (Package, Core, or DRAM).
* **`energy_uj`**: This is the core metric. It provides the current energy consumption in microjoules (µj).
* **`max_energy_range_uj`**: This file gives the maximum value before the counter wraps back to zero.

### Overflow handling

Because the RAPL energy counters are stored in hardware registers with finite bit-widths (typically 32-bit or 64-bit depending on the architecture), they will eventually reach their maximum value and **wrap around** (overflow) back to zero.

The `max_energy_range_uj` file to indicate this threshold. To ensure accurate measurements, especially for long-running benchmarks, the monitoring tool must implement a robust overflow detection and correction logic.

To handle these overflows, the measurement worker thread performs **frequent polling** of the `energy_uj` files. By sampling the counters at a rate significantly higher than the theoretical minimum time it takes for a counter to wrap around, we can safely detect an overflow and correct it. The polling rate must be higher than the minimal period of an overflow, otherwise, an overflow could not be always detected.
In the future, we might implement an overflow period to minimize polling and reduce the overhead it introduces, even so it is not huge .

## Limitations

- Although RAPL interface provides multiple domains enabling fine-grained energy profiling, it does not offer per-process energy attribution, making it difficult to accurately assess the energy consumption of individual processes.
- Some domains like **DRAM** or **PSYS** might not be available and the **Uncore** domain may not include the same components depending on the CPU generation, moreover, the **DRAM** domains might not be included in the **Package** domain.
- Short lived events with variations and quick workloads might not be captured due to the limited resolution of hardware counters.

[^dissecting_software-based_measurement]: Guillaume Raffin, Denis Trystram. Dissecting the software-based measurement of CPU energy consumption: a comparative analysis. IEEE Transactions on Parallel and Distributed Systems, 2024, 36 (1), pp.96. ⟨10.1109/TPDS.2024.3492336⟩. ⟨hal-04420527v3⟩

[^powercap]: [Powercap documentation](https://docs.kernel.org/power/powercap/powercap.html)