# Minimal Example

This guide shows the minimal example to use **Joule Profiler**.

## Minimal Measurement

Measure any program's energy consumption:

```bash
# Measure a simple command
sudo joule-profiler profile -- sleep 1

# Measure a Python script
sudo joule-profiler profile -- python my_script.py

# Measure a compiled program with arguments
sudo joule-profiler profile -- ./my-program arg1 arg2
```

## Basic Phase Detection

Add phase markers to your program:

**Python example:**
```python
# my_script.py
import time

print("__START__", flush=True)
time.sleep(1)
print("__END__", flush=True)
```

The profiler will measure energy separately for the `__START__` to `__END__` phase.