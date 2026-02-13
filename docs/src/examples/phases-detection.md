# Phase Detection

This guide shows how to add phase markers to your programs for energy profiling.

## Basic Concept

Print tokens to stdout and detect different sections of your code. The profiler detects these tokens and measures each section separately.

## Add Phase Markers

Here are several examples in different languages:

**Python:**
```python
print("__INIT__", flush=True)
load()

print("__PROCESSING__", flush=True)
process()
```

**C:**
```c
printf("__INIT__\n");
fflush(stdout);
load();

printf("__PROCESSING__\n");
fflush(stdout);
process();
```

**Rust:**
```rust
println!("__INIT__");
std::io::stdout().flush().unwrap();
load();

println!("__PROCESSING__");
std::io::stdout().flush().unwrap();
process();
```

> [!IMPORTANT]
> Always flush stdout immediately after printing tokens, buffered output may not be detected in time.

## Choose a Token Pattern

Create a regex pattern that matches your tokens:

```bash
# Simple pattern matching __WORD__
--token-pattern "__[A-Z_]+__"

# Custom pattern matching [WORD]
--token-pattern "\[A-Z_]+\]"

# Specific tokens only
--token-pattern "INIT|WORK|END"
```

Common patterns:
- `__[A-Z_]+__` - Matches `__INIT__`, `__WORK__`, `__END__`
- `<<<.*>>>` - Matches `<<<phase1>>>`, `<<<phase2>>>`
- `\[PHASE-[0-9]+\]` - Matches `[PHASE-1]`, `[PHASE-2]`

## Run the Profiler

```bash
sudo joule-profiler phases --token-pattern "__[A-Z_]+__" -- python my_script.py
```

The profiler will:
1. Monitor your program's stdout
2. Detect tokens matching the pattern
3. Measure energy between each token
4. Report energy per phase

## Complete Example

**script.py:**
```python
import time

print("__LOAD__", flush=True)
data = [i for i in range(1000000)]
time.sleep(0.5)

print("__COMPUTE__", flush=True)
result = sum(data)
time.sleep(0.5)

print("__DONE__", flush=True)
```

**Command:**
```bash
sudo joule-profiler phases --token-pattern "__[A-Z_]+__" -- python script.py
```

**Output:**
```
Phase 0: __LOAD__ → __COMPUTE__
  Duration: 502 ms
  package-0: 1.2 J

Phase 1: __COMPUTE__ → __DONE__
  Duration: 501 ms
  package-0: 1.5 J
```

## Best Practices

**Flushing:**
```python
# Good - flushed immediately
print("__PHASE__", flush=True)

# Bad - buffered, may be delayed
print("__PHASE__")
```

**Regex Escaping:**
```bash
# Correct - brackets escaped
--token-pattern "\[WORK\]"

# Wrong - brackets not escaped
--token-pattern "[WORK]"
```

---

If you encounter some issues with phases, see [troubleshooting](../troubleshooting/overview.md).