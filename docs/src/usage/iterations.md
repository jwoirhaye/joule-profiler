# Iterations

Running multiple iterations allows you to **improve measurement accuracy** and **reduce variance**.

Each iteration repeats the profiling session of your program:

- Phases are measured separately for each iteration.
- Metrics are collected independently.
- Aggregated results can be compared across iterations.

## Why use multiple iterations?

- CPU and GPU power measurements can fluctuate due to background processes.
- Running multiple iterations allows you to get a **stable average** and reduce variance between measurements.
- It can help smoothing the profiler warmup overhead.

## Example Commands

```bash
# Run 3 iterations of a program
sudo joule-profiler phases --iterations 3 -- ./my-program
```

> [!NOTE]
> Iteration data is included in all output formats (Terminal, JSON, CSV).
> Each iteration repeats the same phase structure, making comparison straightforward.
> For full examples of iterations and outputs, see the different outputs [examples](../examples/outputs/outputs.md).