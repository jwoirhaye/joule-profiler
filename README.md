# Joule Profiler ⚡

[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Linux](https://img.shields.io/badge/Linux-FCC624?style=for-the-badge&logo=linux&logoColor=black)](https://www.linux.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg?style=for-the-badge)](LICENSE)

**Joule Profiler** is a command-line tool for measuring the energy consumption of programs on Linux systems with Intel processors. It leverages Intel's **RAPL (Running Average Power Limit)** interface to provide accurate, hardware-based energy measurements.

> 💡 **Inspired by [JouleIt](https://github.com/powerapi-ng/jouleit)** - A Rust implementation with enhanced features for energy profiling.

## ✨ Features

- 🔋 **Accurate Energy Measurement**: Hardware-based energy counters via Intel RAPL
- 📊 **Multiple Power Domains**: Measure CPU package, cores, DRAM, and more
- 🎯 **Phase-Based Profiling**: Break down energy consumption by program phases
- 🔄 **Multiple Iterations**: Run programs multiple times for statistical analysis
- 🗂️ **Multiple Output Formats**: Terminal (pretty print), JSON, or CSV
- 🖥️ **Multi-Socket Support**: Works with systems having multiple CPU sockets
- 📝 **Comprehensive Logging**: Detailed logs for debugging and analysis (`-v`, `-vv`, `-vvv`)

## 📋 Table of Contents

- [Requirements](#requirements)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Usage](#usage)
    - [List RAPL Domains](#list-rapl-domains)
    - [Simple Mode](#simple-mode)
    - [Phases Mode](#phases-mode)
    - [Multiple Iterations](#multiple-iterations)
- [Configuration](#configuration)
- [Output Formats](#output-formats)
- [Examples](#examples)
- [How It Works](#how-it-works)
- [Troubleshooting](#troubleshooting)
- [License](#license)
- [Acknowledgments](#acknowledgments)

## 🔧 Requirements

### Hardware
- **Intel CPU** with RAPL support (most Intel CPUs since Sandy Bridge, 2011)
- Linux kernel with `intel_rapl` support (kernel 3.13+)

### Software
- **Linux** operating system
- **Rust** 1.70+ (for building from source)
- Root privileges or appropriate permissions to read RAPL counters

## 📦 Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/jwoirhaye/joule-profiler.git
cd joule-profiler

# Build release binary
cargo build --release

# The binary will be available at target/release/joule-profiler
```

### Using Cargo

```bash
cargo install --path .
```

## 🚀 Quick Start

### Basic Usage

Measure energy consumption of a command:

```bash
# Simple measurement (pretty terminal output)
sudo joule-profiler simple -- ./my-program arg1 arg2

# With JSON output
sudo joule-profiler simple --json -- ./my-program

# With CSV output
sudo joule-profiler simple --csv -- ./my-program
```

### With Logging

```bash
# Info-level logs (-v)
sudo joule-profiler -v simple -- ./my-program

# Debug logs (-vv) for detailed information
sudo joule-profiler -vv simple -- ./benchmark

# Trace logs (-vvv) for maximum verbosity
sudo joule-profiler -vvv simple -- ./my-program
```

## 📖 Usage

### List RAPL Domains

Discover available energy measurement domains on your system:

```bash
sudo joule-profiler list-domains
```

**Example output:**
```
Available RAPL domains:

Socket 0:

  NAME             RAW_NAME             PATH
  ----             --------             ----
  CORE             core                 /sys/devices/virtual/powercap/intel-rapl/intel-rapl:0/intel-rapl:0:0/energy_uj
  PACKAGE-0        package-0            /sys/devices/virtual/powercap/intel-rapl/intel-rapl:0/energy_uj
  UNCORE           uncore               /sys/devices/virtual/powercap/intel-rapl/intel-rapl:0/intel-rapl:0:1/energy_uj

Socket 1:

  NAME             RAW_NAME             PATH
  ----             --------             ----
  PSYS             psys                 /sys/devices/virtual/powercap/intel-rapl/intel-rapl:1/energy_uj


```

### Simple Mode

Measure total energy consumption of a program:

```bash
joule-profiler simple [OPTIONS] -- COMMAND [ARGS]
```

**Options:**
- `--json`: Export results as JSON instead of terminal output
- `--csv`: Export results as CSV (semicolon-separated values)
- `-n, --iterations <N>`: Number of times to run the measurement (>=1)
- `--jouleit-file <FILE>`: Output file for CSV/JSON (default: `data<TIMESTAMP>.csv/json`)
- `-s, --sockets <SOCKETS>`: Sockets to measure (e.g., `0` or `0,1`)
- `-o, --output-file <FILE>`: Redirect profiled program's stdout to file

**Examples:**

```bash
# Terminal output (default - pretty printed)
sudo joule-profiler simple -- python script.py

# JSON output
sudo joule-profiler simple --json -- ./compute-app

# CSV output with custom filename
sudo joule-profiler simple --csv --jouleit-file results.csv -- ./my-program

# Measure specific sockets only
sudo joule-profiler simple --sockets 0 -- ./my-program

# Save program output to file
sudo joule-profiler simple --output-file output.txt -- ./my-program
```

**Terminal Output Example:**

```
═══════════════════════════════════════
  Energy consumption (Joules)
═══════════════════════════════════════
  CORE_0              :   0.756590 J
  PACKAGE-0_0         :   3.639090 J
  DRAM_0              :   0.845123 J
  UNCORE_0            :   0.003784 J
───────────────────────────────────────
  Total energy (J):   5.244587
  Average power (W):      5.237
  Duration (s)    :      1.002
  Exit code       : 0
═══════════════════════════════════════
```

### Phases Mode

Measure energy consumption of specific phases within a program by detecting output tokens:

```bash
joule-profiler phases [OPTIONS] -- COMMAND [ARGS]
```

**Options:**
- `--token-start <TOKEN>`: Start token printed by the program on stdout [default: `__WORK_START__`]
- `--token-end <TOKEN>`: End token printed by the program on stdout [default: `__WORK_END__`]
- `--json`: Export results as JSON (default: terminal pretty print)
- `--csv`: Export results as CSV (semicolon-separated values)
- `-n, --iterations <N>`: Number of iterations (>=1)
- `--jouleit-file <FILE>`: Output file for CSV/JSON (else `data<TIMESTAMP>.csv/json`)
- `-s, --sockets <SOCKETS>`: Sockets to measure
- `-o, --output-file <FILE>`: Redirect profiled program's stdout to file

**How it works:**
1. The tool monitors your program's stdout
2. When `token-start` is detected, it records the starting energy
3. When `token-end` is detected, it records the ending energy
4. Energy consumption is calculated for multiple phases:
    - **Global**: Total program execution (START → END)
    - **Pre-work**: From start to `token-start`
    - **Work**: From `token-start` to `token-end`
    - **Post-work**: From `token-end` to program end

**Example program output:**

```python
# example.py
print("Initializing...")
print("Loading data...")
print("__WORK_START__")  # Default token
# Heavy computation here
print("Processing complete")
print("__WORK_END__")    # Default token
print("Cleaning up...")
```

**Measurement command:**

```bash
# Using default tokens
sudo joule-profiler phases -- python example.py

# Using custom tokens
sudo joule-profiler phases \
    --token-start "=== BEGIN ===" \
    --token-end "=== DONE ===" \
    -- python3 example.py

# With JSON output
sudo joule-profiler phases --json -- python example.py
```

**Terminal Output Example:**

```
═══════════════════════════════════════
  Phase: global (START -> END)
═══════════════════════════════════════
  CORE_0              :   2.456789 J
  PACKAGE-0_0         :  12.345678 J
  DRAM_0              :   3.890123 J
───────────────────────────────────────
  Total energy (J):  18.692590
  Average power (W):     17.234
  Duration (s)    :      1.085
  Exit code       : 0

═══════════════════════════════════════
  Phase: pre_work (START -> __WORK_START__)
═══════════════════════════════════════
  CORE_0              :   0.123456 J
  PACKAGE-0_0         :   1.234567 J
  DRAM_0              :   0.345678 J
───────────────────────────────────────
  Total energy (J):   1.703701
  Average power (W):      5.678
  Duration (s)    :      0.300

═══════════════════════════════════════
  Phase: work (__WORK_START__ -> __WORK_END__)
═══════════════════════════════════════
  CORE_0              :   2.100000 J
  PACKAGE-0_0         :  10.000000 J
  DRAM_0              :   3.200000 J
───────────────────────────────────────
  Total energy (J):  15.300000
  Average power (W):     20.400
  Duration (s)    :      0.750

═══════════════════════════════════════
  Phase: post_work (__WORK_END__ -> END)
═══════════════════════════════════════
  CORE_0              :   0.233333 J
  PACKAGE-0_0         :   1.111111 J
  DRAM_0              :   0.344445 J
───────────────────────────────────────
  Total energy (J):   1.688889
  Average power (W):     48.254
  Duration (s)    :      0.035
═══════════════════════════════════════
```

### Multiple Iterations

Run measurements multiple times for statistical analysis:

```bash
sudo joule-profiler simple --iterations 10 -- ./my-program
```

This will:
- Execute the program 10 times
- Measure energy for each iteration
- Display results for each run

**Example:**

```bash
# 5 iterations with JSON output
sudo joule-profiler simple --iterations 5 --json --jouleit-file stats.json -- ./my-program

# 10 iterations in phases mode
sudo joule-profiler phases --iterations 10 --csv -- ./my-program
```

### Global Options

```bash
joule-profiler [OPTIONS] <COMMAND>
```

**Options:**
- `-v, --verbose...`: Verbosity (-v, -vv, -vvv)
- `--rapl-path <PATH>`: Override default RAPL base path (default: `/sys/devices/virtual/powercap/intel-rapl`)
- `-h, --help`: Print help
- `-V, --version`: Print version

### Environment Variables

- `JOULE_PROFILER_RAPL_PATH`: Override default RAPL base path

**Example:**

```bash
# Use custom RAPL path
export JOULE_PROFILER_RAPL_PATH=/custom/path/to/rapl
sudo joule-profiler simple -- ./my-program

# Or inline
sudo JOULE_PROFILER_RAPL_PATH=/custom/path joule-profiler simple -- ./my-program
```

---

## 💡 Examples

### Example 1: Quick Energy Check

```bash
# Measure a simple command
sudo joule-profiler simple -- sleep 1

# With logging
sudo joule-profiler -v simple -- sleep 1
```

### Example 2: Python Script

```bash
# Measure a Python script
sudo joule-profiler simple -- python train_model.py

# With phases
sudo joule-profiler phases \
    --token-start "Training started" \
    --token-end "Training complete" \
    -- python3 train_model.py

# Save results to JSON
sudo joule-profiler simple --json --jouleit-file training-energy.json -- python train_model.py
```
### Example 3: Benchmark

```bash
# Measure benchmark
sudo joule-profiler simple -- ./benchmark --threads 16

# Specific sockets
sudo joule-profiler simple --sockets 0,1 -- ./benchmark

# Save benchmark output
sudo joule-profiler simple --output-file benchmark.log -- ./benchmark
```

### Example 4: Container/Docker

```bash
# Measure Docker container
sudo joule-profiler simple -- docker run image-name 
```

### Example 5: Phase-Based Analysis

```bash
# C program with custom tokens
sudo joule-profiler phases \
    --token-start "COMPUTATION_START" \
    --token-end "COMPUTATION_END" \
    --json \
    -- ./scientific-simulation

# Multiple iterations for reliability
sudo joule-profiler phases --iterations 10 --csv -- ./my-app
```

## 🔍 How It Works

Joule Profiler measures energy by:

1. **Reading RAPL counters** before program execution
2. **Running your program** to completion
3. **Reading RAPL counters** after program execution
4. **Computing the difference** (handling counter wraparound)
5. **Reporting energy per domain** (CPU, DRAM, etc.)

### Power Domains

- **PACKAGE**: Total CPU package energy (cores + cache + integrated GPU)
- **CORE**: CPU cores only
- **UNCORE**: Uncore components (L3 cache, memory controller, iGPU on some CPUs)
- **DRAM**: Memory energy
- **PSYS**: Platform energy (entire system, available on some laptops)

### RAPL Interface

RAPL counters are exposed via the Linux powercap framework at:
```
/sys/devices/virtual/powercap/intel-rapl/
```

Each domain has:
- `energy_uj`: Current energy counter in microjoules (µJ)
- `max_energy_range_uj`: Maximum counter value before wraparound

## 🐛 Troubleshooting

### Permission Denied

**Problem:** Cannot read RAPL counters

**Solutions:**

```bash
sudo joule-profiler simple -- ./my-program
```

### No Domains Found

**Problem:** `No RAPL domains found`

**Check available domains:**
```bash
sudo joule-profiler list-domains
```

**Solutions:**

- Update Linux kernel to 3.13+
- Check BIOS settings (some systems disable RAPL)
- Verify CPU supports RAPL (most Intel CPUs since 2011)
- Try specifying custom RAPL path:
  ```bash
  sudo joule-profiler --rapl-path /sys/class/powercap/intel-rapl simple -- ./my-program
  ```

### Inaccurate Measurements

**Problem:** Energy measurements vary significantly

**Solutions:**

```bash
# Use multiple iterations
sudo joule-profiler simple --iterations 10 -- ./my-program

# Enable logging to see warnings
sudo joule-profiler -vv simple -- ./my-program

# Minimize background processes
# Close browsers, IDEs, etc.

# Disable CPU frequency scaling (optional)
sudo cpupower frequency-set --governor performance

# Measure longer-running programs (>1 second)
```

### Tokens Not Detected (Phases Mode)

**Problem:** Phases not computed, warnings about missing tokens

**Check with logging:**
```bash
sudo joule-profiler -v phases -- ./my-program
```

**Solutions:**

1. Verify tokens are printed to **stdout** (not stderr)
2. Check token spelling (case-sensitive, default: `__WORK_START__` and `__WORK_END__`)
3. Ensure tokens are **always printed** (not conditional)
4. Flush output buffers in your program:
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

### Counter Overflow Warning

**Problem:** Warning about energy counter at 90%+ of max range

**Explanation:** This is informational. The tool handles counter overflow automatically, but warns you when a counter is close to wrapping around.

**Solution:** This is normal for long-running systems. The measurement remains accurate.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [JouleIt](https://github.com/powerapi-ng/jouleit) by [@powerapi-ng](https://github.com/powerapi-ng) for inspiration
- Intel for the RAPL technology
- Linux kernel developers for the powercap framework

## 📚 Further Reading

- [Linux Powercap Framework](https://www.kernel.org/doc/html/latest/power/powercap/powercap.html)
- [Intel RAPL Documentation](https://www.intel.com/content/www/us/en/developer/articles/technical/software-security-guidance/advisory-guidance/running-average-power-limit-energy-reporting.html)
- [JouleIt - Original Project](https://github.com/powerapi-ng/jouleit)
- [IEEE Paper - Dissecting the Software-Based Measurement of CPU Energy Consumption](https://ieeexplore.ieee.org/document/10746340/)

## 📧 Contact

For questions, issues, or suggestions:
- Open an issue on [GitHub](https://github.com/jwoirhaye/joule-profiler/issues)
- GitHub: [@jwoirhaye](https://github.com/jwoirhaye)

---

**Inspired by [JouleIt](https://github.com/powerapi-ng/jouleit) 💚**