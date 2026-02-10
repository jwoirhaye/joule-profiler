# Measurement

Joule Profiler measures program execution in terms of **phases** and **metrics**.  

A measurement captures how your program consumes energy and resources during a specific interval.

## Phases

A **phase** is a logical section of your program you want to profile.  

- By default, the profiler creates a single phase: `START -> END`.  
- You can define additional phases by inserting **custom tokens** in your program output.  

Phases help you isolate different parts of your program for detailed analysis.

## Metrics

During a phase, Joule Profiler collects metrics from all configured **sources**, such as:

- CPU energy consumption
- DRAM energy consumption
- GPU usage (if Nvidia support is enabled)
- Other supported sensors

> [!NOTE]
> Each source reports values for its sensors, which are aggregated per phase **after the measurements** to reduce the computation overhead.

## Example Commands

```bash
# Simple measurement with default terminal output
sudo joule-profiler phases -- ./my-program

# With GPU measurements enabled
sudo joule-profiler --gpu phases -- ./my-program

# With Powercap RAPL backend (Powercap or perf available)
sudo joule-profiler --rapl-backend=powercap phases -- ./my-program
```

For more advanced usages, see the provided [examples](../examples/overview.md).