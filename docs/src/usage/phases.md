# Phases

## Overview

Phase-based profiling is the key feature that distinguishes **Joule Profiler** from other energy profilers. It enables energy measurement of specific sections of your program, allowing you to identify which parts contribute most to energy consumption.

## Why Use Phases?

### Identify Energy Hotspots

Instead of measuring total program energy, phases let you pinpoint expensive sections:

```python
print("__INIT__", flush=True)
loading()  # How much loading consume?

print("__WORK__", flush=True)
for batch in data:
    process(batch)  # vs actual work

print("__CLEANUP__", flush=True)
save_results()
```

Each phase appears separately in the results, showing its individual energy consumption.

### Exclude Initialization Overhead

Interpreted languages (Python, JavaScript, Ruby) have significant startup overhead that dominates short programs:

```bash
# Python interpreter initialization can be 50-100ms
# Your actual work might be only 10ms
sudo joule-profiler simple -- python my_script.py
# Result: 90% of energy is interpreter startup, not your code
```

If you want to exclude this initialization phase, then using phases might be a good answer:

```python
# my_script.py
import heavy_libraries
model = load_model()  # Initialization overhead

print("__START__", flush=True)  # Mark the beginning of actual work
result = run_inference(data)    # This is what you want to measure
print("__END__", flush=True)

save_output(result)
```

```bash
sudo joule-profiler phases --token-pattern "__START__|__END__" -- python my_script.py
```

Now you measure the `__START__` to `__END__` phase, excluding interpreter startup and library loading.

## Implementation Notes

Phases are detected by monitoring the program's stdout for tokens matching a regular expression.

The profiler:
1. Spawns your program with stdout captured
2. Scans each output line for the phase regex
3. Triggers measurements when tokens are detected
4. Associates energy deltas with the correct phase

Future versions may support lower-overhead mechanisms like inter-process communication with language-specific wrappers to minimize the overhead introduced by I/O operations.