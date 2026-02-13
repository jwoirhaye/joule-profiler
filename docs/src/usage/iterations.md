# Iterations

Running multiple iterations allows you to **improve measurement accuracy** and **reduce variance**.
Each iteration repeats the profiling session of your program:
- Phases are measured separately for each iteration.
- Metrics are collected independently.
- Aggregated results can be compared across iterations.

## Why use multiple iterations?

**Reduce measurement variance:**
- CPU and GPU power measurements can fluctuate due to background processes.
- Running multiple iterations allows you to get a **stable average** and reduce variance between measurements.
- It helps smooth out the profiler's own warmup overhead.

**Account for machine warmup:**
- The first run of a program often behaves differently due to:
  - CPU frequency scaling (performance governor scaling up)
  - Cold caches
  - Thermal state (cooler CPU initially, then warming up)
  - OS resource allocation

If your program doesn't include its own warmup phase, the first iteration may show significantly different energy consumption than subsequent ones.

You can then either:
- Exclude the first iteration from your analysis
- Ensure all benchmarks include this warmup effect for fair comparison
- Add an explicit warmup phase before measurement

## Example Commands

```bash
# Run 5 iterations
sudo joule-profiler phases --iterations 5 -- ./my-program
```

> [!NOTE]
> Iteration data is included in all output formats (Terminal, JSON, CSV).
> Each iteration repeats the same phase structure, making comparison straightforward.
> For full examples of iterations and outputs, see the different outputs [examples](../examples/outputs/outputs.md).