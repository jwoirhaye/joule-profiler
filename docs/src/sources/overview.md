## Sources Overview

**Joule Profiler** is designed to be simple, portable, and focused on monitoring the components responsible for the majority of energy consumption, such as the CPU, GPU, and SoC. While it does not aim to support every possible energy source on every device, the profiler can be extended by implementing custom sources for specific devices or components.  

### Supported Architectures

- **CPU:** The primary target is x86, with future plans for selected ARM processors.  
- **GPU:** Current focus is on Nvidia GPUs.  
- **OS:** Only Linux-based systems are officially supported at the moment, but support for Windows and macOS is a potential future extension.  

### Available Sources

- **Intel RAPL:** Measures CPU and DRAM energy domains.  
  - Implemented using either **perf_events** or **Powercap** on Linux systems.  
  - For details, see [RAPL Overview](rapl/introduction.md).  

- **Nvidia GPUs (NVML):** Provides energy and performance metrics for Nvidia GPUs.  
  - Currently tested on Linux; should work on Windows, but not fully verified.  
  - For details, see [NVML Overview](nvml/introduction.md).  

### Extending Joule Profiler

Users can implement new metric sources allowing the monitoring of additional devices or components beyond the default set. For guidance, see [Adding a New Source](../developer-guide/adding-source/overview.md).  
