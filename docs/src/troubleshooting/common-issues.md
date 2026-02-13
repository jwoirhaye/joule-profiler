# Common Issues

## Permission Denied

**Problem:** Cannot read RAPL counters or perf event.

**Solutions:** Run with root privileges when using powercap backend or configure [perf_event_paranoid configuration](../sources/perf_event/perf_event_paranoid.md) for perf_event backend.

## No RAPL Domains Found

**Problem:** `No RAPL domains found`.

This issue happens with **powercap** backend.

**Check available sensors:**
```bash
sudo joule-profiler list-sensors
```

**Solutions:**
- Update Linux kernel to 3.13+ (required for RAPL support)
- Check BIOS settings (some systems disable RAPL in firmware)
- Verify CPU supports RAPL (most Intel CPUs since Sandy Bridge/2011, AMD since Zen)
- Try specifying custom RAPL path:
  ```bash
  sudo joule-profiler --rapl-path /sys/class/powercap/intel-rapl phases -- ./my-program
  ```

## High Variance in Measurements

**Problem:** Energy measurements vary significantly between iterations.

### Why This Happens

**Short Program Duration**: For programs running under 100ms, measurement overhead and system noise become proportionally significant:
- Profiler initialization (spawning, pausing, counter attachment)
- OS scheduling decisions and cache state variations
- Hardware counter granularity relative to execution time
- It also depends on the hardware energy management, on laptops the variance is significantly higher than on desktop computers, due to the energy saving policies. 

**System Activity**: Background processes, thermal throttling, and frequency scaling cause variations.

### Solutions

```bash
# 1. Use multiple iterations to reduce variance
sudo joule-profiler sphasese -n 20 -- ./my-program

# 2. Enable logging to see warnings and diagnostics
sudo joule-profiler -vv phases -- ./my-program

# 3. Minimize background processes
# Close browsers, IDEs, file syncing, etc.

# 4. Disable CPU frequency scaling (optional, for more stable results)
sudo cpupower frequency-set --governor performance

# 5. Profile longer-running programs
# Extend your workload or loop it internally:
./my-program --repeat 1000
```

**For Short Programs**: If your program must run quickly, increase internal iteration count rather than relying on profiler iterations.

## Tokens Not Detected (Phases Mode)

**Problem:** No phases computed, warning "No tokens matching pattern".

**Check with logging:**
```bash
sudo joule-profiler -v phases -- ./my-program
```

**Solutions:**

**Verify tokens are printed to stdout** (not stderr):
```python
# Correct (stdout)
print("__INIT__")

# Wrong (stderr)
import sys
print("__INIT__", file=sys.stderr)
```

## Incorrect phases measures

**Solution:** Flush the stdout on each print, otherwise the prints will be buffered and the phases will be merged.

Example output without stdout flushing:
```
╔════════════════════════════════════════════════╗
║  Phase: START -> __WORK_START__                ║
╚════════════════════════════════════════════════╝
  Duration            :        109 ms
  Start token         :      START
  End token           : __WORK_START__ (line 1)

┌────────────────────────────────────────────────┐
│ RAPL (perf_event)                              │
└────────────────────────────────────────────────┘
  CORE-0              :    9317199 µJ
  DRAM-0              :     126281 µJ
  PACKAGE-0           :   10607910 µJ

╔════════════════════════════════════════════════╗
║  Phase: __WORK_START__ -> __WORK_END__         ║
╚════════════════════════════════════════════════╝
  Duration            :          0 ms
  Start token         : __WORK_START__ (line 1)
  End token           : __WORK_END__ (line 2)

┌────────────────────────────────────────────────┐
│ RAPL (perf_event)                              │
└────────────────────────────────────────────────┘
  CORE-0              :       5187 µJ
  DRAM-0              :          0 µJ
  PACKAGE-0           :          0 µJ
```

We can see that the **__WORK_START__ -> __WORK_END__** phase is during 0ms, which is the longest running phase.
To fix this issue, we need to flush the stdout on each print to avoid buffering:

```python
# Python
print("__WORK_START__", flush=True)
# ... work ...
print("__WORK_END__", flush=True)
```

```rust
// Rust
println!("__WORK_START__");
use std::io::{self, Write};
io::stdout().flush().unwrap();
```

```c
// C
printf("__WORK_START__\n");
fflush(stdout);
```

Which gives the following results:

```
════════════════════════════════════════════════╗
║  Phase: START -> __WORK_START__                ║
╚════════════════════════════════════════════════╝
  Duration            :         25 ms
  Start token         :      START
  End token           : __WORK_START__ (line 1)

┌────────────────────────────────────────────────┐
│ RAPL (perf_event)                              │
└────────────────────────────────────────────────┘
  CORE-0              :    2244689 µJ
  DRAM-0              :      43273 µJ
  PACKAGE-0           :    2518737 µJ

╔════════════════════════════════════════════════╗
║  Phase: __WORK_START__ -> __WORK_END__         ║
╚════════════════════════════════════════════════╝
  Duration            :         77 ms
  Start token         : __WORK_START__ (line 1)
  End token           : __WORK_END__ (line 2)

┌────────────────────────────────────────────────┐
│ RAPL (perf_event)                              │
└────────────────────────────────────────────────┘
  CORE-0              :    6755859 µJ
  DRAM-0              :      80505 µJ
  PACKAGE-0           :    7568481 µJ
```

Here, we can see that the **__WORK_START__ -> __WORK_END__** has more accurate results due to stdout flushing.

## Invalid Regex Pattern

**Problem:** Error "Invalid regex pattern"

**Common mistakes:**
```bash
# Wrong: unescaped special characters
--token-pattern "[INIT]"

# Correct: escape brackets
--token-pattern "\[INIT\]"

# Wrong: unclosed group
--token-pattern "(INIT"

# Correct: balanced parentheses
--token-pattern "(INIT)"
```

**Solution:** Test your regex pattern before profiling:
- Use online tools: https://regex101.com/
- Test with grep: `echo "your output" | grep -E "your-pattern"`

## Counter Overflow Warning

**Problem:** Warning about energy counter at 90%+ of max range on RAPL powercap backend.

**Explanation:** This is informational, not an error. Hardware energy counters have a maximum value before they wrap around. The profiler detects when counters approach this limit and handles overflow automatically.

**Solution:** This is normal for long-running systems or high power consumption. The measurement remains accurate - the warning is just informing you that internal counter wrapping was handled.

## Missing Metrics

**Problem:** Expected metrics don't appear in results.

**Solutions:**
- Verify your hardware supports the requested counters (e.g., RAPL, perf_event support)
- Check that metric sources are properly initialized (use `-vv` for detailed logging)
- Ensure you have necessary permissions for the metric source
- List available sensors: `sudo joule-profiler list-sensors`

## High Overhead on Very Frequent Measurements

**Problem:** Profiler seems to slow down the program significantly

**Cause:** Too many phase transitions (measuring every few milliseconds)

**Solution:** Reduce measurement frequency:
- Use phases only for major sections (not every small operation)
- Batch work between phase tokens
- Aim for phases lasting at least 100ms each