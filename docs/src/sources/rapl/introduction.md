# RAPL

## Overview

RAPL (Running Average Power Limit) is an Intel processor feature that allows real-time energy consumption measurements of CPU and memory subsystem.

This technology has been available on Intel processors since *Sandy Bridge* generation.

RAPL provides energy measurements at different scales, enabling you to measure energy consumption per component and understand more precisely how each part of the system contributes to the overall power usage.

It allows fine-grained energy profiling of CPU cores, memory subsystem, and uncore components.

## Architecture

**RAPL** interface exposes multiple power domains that allow measuring energy consumption of different parts of the processor and memory subsystem.
Domains metrics are accessible through model-specific registers (MSRs) on the host system, enabling user to monitor power usage in real time.

### Domains

| Domain | Description |
|--------|------------|
| **Package/PKG** | Entire CPU socket. This includes cores and uncore components. |
| **Core/PP0** | Represents the CPU cores only. Useful for profiling per-core energy consumption. |
| **Uncore/PP1** | Covers the energy consumption of the integrated GPU if available. |
| **DRAM** | Dynamic random access memory attached to the integrated memory controller if supported. |
| **PSYS** | Entire SoC energy consumption, unique (only one PSYS for the entire SoC at most, available since *Skylake* generation) |

Architecture of RAPL[^dissecting_software-based_measurement]:

![RAPL architecture](../../figures/rapl_architecture.png)

> [!NOTE]
> - Some of the domains may not appear depending on the processor architecture.
> - The **PSYS** domain can report the same consumption as an external wattmeter [^dissecting_software-based_measurement], representing the entire computer consumption. These results, obtained on a laptop, should be interpreted with caution and could not reflect the real world.

[^dissecting_software-based_measurement]: G. Raffin and D. Trystram, "Dissecting the Software-Based Measurement of CPU Energy Consumption: A Comparative Analysis," in IEEE Transactions on Parallel and Distributed Systems, vol. 36, no. 1, pp. 96-107, Jan. 2025, doi: 10.1109/TPDS.2024.3492336.
