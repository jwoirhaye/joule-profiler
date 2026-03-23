## Sources Overview

**Joule Profiler** is designed to be simple, portable, and focused on monitoring the components responsible for the majority of energy consumption, such as the CPU, GPU, and SoC. While it does not aim to support every possible energy source on every device, the profiler can be extended by implementing custom sources for specific devices or components.  

### Supported Architectures

- **CPU:** The only target is Intel x86 architecture at the moment.  
- **GPU:** Current focus is on Nvidia GPUs.
- **OS:** Only Linux-based systems are officially supported at the moment.  

### Available Sources

- **Intel RAPL:** Measures CPU and DRAM energy domains.  
  - Implemented using either **perf_event** or **Powercap** on Linux systems.  
  - For details, see [RAPL Overview](rapl/introduction.md).  

- **Nvidia GPUs (NVML):** Provides energy and performance metrics for Nvidia GPUs.  
  - NVML is available on Linux and Windows, but we do not support Windows systems.  
  - For details, see [NVML Overview](nvml/introduction.md).  

- **perf_event:** Measures various performance counters like hardware, software, kernel probes, etc.
  - Available only on Linux.
  - For details, see [perf_event Overview](perf_event/introduction.md).

### Extending Joule Profiler

Users can implement new metric sources allowing the monitoring of additional devices or components beyond the default set. For guidance, see [Adding a New Source](../developer-guide/adding-source/overview.md).  
