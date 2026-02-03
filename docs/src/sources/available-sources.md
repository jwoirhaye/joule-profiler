## Targeted Environments

The goal of **Joule Profiler** is not to support every possible energy source on every device or architecture. Instead, it is designed to remain simple, portable, and focused on providing information on the components responsible for the majority of energy consumption, such as the CPU, GPU, and SoC. Users can also implement their own sources, allowing the profiler to be extended for specific devices or components as needed.

The primary target architecture is x86, with plans to add support for some ARM processors in the future. For GPUs, the focus is currently on Nvidia GPUs.

At present, only Linux-based systems are supported, but future support for Windows and macOS would be desirable.

## Available Sources

For now, [RAPL](rapl/rapl.md) domains support is implemented using [perf_events](rapl/perf_events.md) or [Powercap](rapl/powercap.md) on Linux systems.
[Nvidia GPU](nvml.md) support is available on Linux and should also work on Windows, although this has not yet been tested.