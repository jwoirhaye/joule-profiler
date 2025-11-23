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

### Quick Install (Recommended)

Install the latest version with a single command:

```bash
curl -fsSL https://raw.githubusercontent.com/jwoirhaye/joule-profiler/main/install.sh | bash
```

### Custom Installation

```bash
# Install to custom directory
curl -fsSL https://raw.githubusercontent.com/jwoirhaye/joule-profiler/main/install.sh | bash -s -- --dir ~/.local/bin

# Install specific version
curl -fsSL https://raw.githubusercontent.com/jwoirhaye/joule-profiler/main/install.sh | bash -s -- --version v0.1.0

# List available versions
curl -fsSL https://raw.githubusercontent.com/jwoirhaye/joule-profiler/main/install.sh | bash -s -- --list

# Non-interactive (for CI/CD)
curl -fsSL https://raw.githubusercontent.com/jwoirhaye/joule-profiler/main/install.sh | bash -s -- --yes
```
### From Source

```bash
# Clone the repository
git clone https://github.com/jwoirhaye/joule-profiler.git
cd joule-profiler

# Build and install system-wide
cargo build --release
sudo cp target/release/joule-profiler /usr/local/bin/

# Verify installation
sudo joule-profiler --version
```

**Note:** System-wide installation (`/usr/local/bin/`) is recommended as the tool requires `sudo` to access RAPL counters.

### Uninstall

```bash
# Using uninstaller
curl -fsSL https://raw.githubusercontent.com/jwoirhaye/joule-profiler/main/uninstall.sh | bash

# Or manually
sudo rm /usr/local/bin/joule-profiler
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
- `--token-pattern <REGEX>`: Regex pattern to detect phase tokens in stdout (default: `__[A-Z0-9_]+__`)
- `--json`: Export results as JSON (default: terminal pretty print)
- `--csv`: Export results as CSV (semicolon-separated values)
- `-n, --iterations <N>`: Number of iterations (>=1)
- `--jouleit-file <FILE>`: Output file for CSV/JSON (else `data<TIMESTAMP>.csv/json`)
- `-s, --sockets <SOCKETS>`: Sockets to measure
- `-o, --output-file <FILE>`: Redirect profiled program's stdout to file

**How it works:**

**Key Concept**: Instead of defining fixed start/end tokens, you provide a **regex pattern** that matches ALL your phase markers. The tool automatically creates phases between consecutive matched tokens.

**Pattern matching:**

- If pattern has a capture group (parentheses), the captured text is used as token name
- Otherwise, the full match is used as token name

**Energy phases computed:**

1. **Global**: Total program execution (START → END)
2. **START -> first_token**: From program start to first matched token
3. **token_i -> token_i+1**: Between each pair of consecutive matched tokens
4. **last_token -> END**: From last matched token to program end

**Default Pattern**

By default, the pattern `__[A-Z0-9_]+__` matches tokens like:

- `__INIT__`
- `__LOAD_DATA__`
- `__COMPUTE__`
- `__CLEANUP__`


**Example program output:**

```python
# example.py
print("Starting program...")
print("__INIT__")
# Initialization code
print("__LOAD_DATA__")
# Load data
print("__COMPUTE__")
# Heavy computation
print("__CLEANUP__")
# Cleanup
print("Done!")
```

**Measurement command:**

```bash
# Using default pattern
sudo joule-profiler phases -- python example.py

# Detected phases:
# - global (START -> END)
# - START -> __INIT__
# - __INIT__ -> __LOAD_DATA__
# - __LOAD_DATA__ -> __COMPUTE__
# - __COMPUTE__ -> __CLEANUP__
# - __CLEANUP__ -> END
```

**Terminal Output Example:**

```
╔══════════════════════════════════════════════════╗
║  Command                                         ║
╚══════════════════════════════════════════════════╝
  python3 example.py

╔══════════════════════════════════════════════════╗
║  Phase: global (START -> END)                    ║
╚══════════════════════════════════════════════════╝
  Start token: START
  End token  : END


══════════════════════════════════════════════════
  Energy consumption (Joules)
══════════════════════════════════════════════════
  CORE_0              :   0.052673 J
  PACKAGE-0_0         :   0.118897 J
  PSYS_1              :   0.361754 J
  UNCORE_0            :   0.000000 J
──────────────────────────────────────────────────
  Total energy        :   0.533324 J
  Average power       :  23.188000 W
  Duration            :   0.023000 s
  Exit code           :          0
══════════════════════════════════════════════════

╔══════════════════════════════════════════════════╗
║  Phase: START -> __INIT__                        ║
╚══════════════════════════════════════════════════╝
  Start token: START
  End token  : __INIT__ (line 2)


══════════════════════════════════════════════════
  Energy consumption (Joules)
══════════════════════════════════════════════════
  CORE_0              :   0.040588 J
  PACKAGE-0_0         :   0.097229 J
  PSYS_1              :   0.303161 J
  UNCORE_0            :   0.000000 J
──────────────────────────────────────────────────
  Total energy        :   0.440978 J
  Average power       :  22.048900 W
  Duration            :   0.020000 s
  Exit code           :          0
══════════════════════════════════════════════════

╔══════════════════════════════════════════════════╗
║  Phase: __INIT__ -> __LOAD_DATA__                ║
╚══════════════════════════════════════════════════╝
  Start token: __INIT__ (line 2)
  End token  : __LOAD_DATA__ (line 3)


══════════════════════════════════════════════════
  Energy consumption (Joules)
══════════════════════════════════════════════════
  CORE_0              :   0.000000 J
  PACKAGE-0_0         :   0.000000 J
  PSYS_1              :   0.000000 J
  UNCORE_0            :   0.000000 J
──────────────────────────────────────────────────
  Total energy        :   0.000000 J
  Average power       :   0.000000 W
  Duration            :   0.000000 s
  Exit code           :          0
══════════════════════════════════════════════════

╔══════════════════════════════════════════════════╗
║  Phase: __LOAD_DATA__ -> __COMPUTE__             ║
╚══════════════════════════════════════════════════╝
  Start token: __LOAD_DATA__ (line 3)
  End token  : __COMPUTE__ (line 4)


══════════════════════════════════════════════════
  Energy consumption (Joules)
══════════════════════════════════════════════════
  CORE_0              :   0.000000 J
  PACKAGE-0_0         :   0.000000 J
  PSYS_1              :   0.000000 J
  UNCORE_0            :   0.000000 J
──────────────────────────────────────────────────
  Total energy        :   0.000000 J
  Average power       :   0.000000 W
  Duration            :   0.000000 s
  Exit code           :          0
══════════════════════════════════════════════════

╔══════════════════════════════════════════════════╗
║  Phase: __COMPUTE__ -> __CLEANUP__               ║
╚══════════════════════════════════════════════════╝
  Start token: __COMPUTE__ (line 4)
  End token  : __CLEANUP__ (line 5)


══════════════════════════════════════════════════
  Energy consumption (Joules)
══════════════════════════════════════════════════
  CORE_0              :   0.000000 J
  PACKAGE-0_0         :   0.000000 J
  PSYS_1              :   0.000000 J
  UNCORE_0            :   0.000000 J
──────────────────────────────────────────────────
  Total energy        :   0.000000 J
  Average power       :   0.000000 W
  Duration            :   0.000000 s
  Exit code           :          0
══════════════════════════════════════════════════

╔══════════════════════════════════════════════════╗
║  Phase: __CLEANUP__ -> END                       ║
╚══════════════════════════════════════════════════╝
  Start token: __CLEANUP__ (line 5)
  End token  : END


══════════════════════════════════════════════════
  Energy consumption (Joules)
══════════════════════════════════════════════════
  CORE_0              :   0.012085 J
  PACKAGE-0_0         :   0.021668 J
  PSYS_1              :   0.058593 J
  UNCORE_0            :   0.000000 J
──────────────────────────────────────────────────
  Total energy        :   0.092346 J
  Average power       :  46.173000 W
  Duration            :   0.002000 s
  Exit code           :          0
══════════════════════════════════════════════════


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

### Example 2: Python Script with Phases

```bash
# Measure with default pattern
sudo joule-profiler phases -- python train_model.py

# Custom pattern for "=== PHASE ===" markers
sudo joule-profiler phases \
    --token-pattern "=== ([A-Z_]+) ===" \
    -- python train_model.py

# Save results to JSON
sudo joule-profiler phases --json --jouleit-file training-energy.json -- python train_model.py
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

### Example 5: Custom Token Format

```bash
# Match timestamps [HH:MM:SS]
sudo joule-profiler phases --token-pattern "\[(\d{2}:\d{2}:\d{2})\]" -- ./program

# Match >>> phase <<< markers
sudo joule-profiler phases --token-pattern ">>> (.*) <<<" -- ./program

# Match underscore-prefixed tokens
sudo joule-profiler phases --token-pattern "_[a-z]+" -- ./program
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

**Problem:** No phases computed, warning "No tokens matching pattern"

**Check with logging:**
```bash
sudo joule-profiler -v phases -- ./my-program
```

**Solutions:**

1. Verify tokens are printed to **stdout** (not stderr)
2. Check tokens are printed to stdout (not stderr):
    ```python
    # Correct (stdout)
    print("__INIT__")
    
    # Wrong (stderr)
    import sys
    print("__INIT__", file=sys.stderr)
   ```
3. Flush output buffers:
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
   
### Invalid Regex Pattern

**Problem:** Error "Invalid regex pattern"

**Solution:** Check regex syntax. Common mistakes:

```bash
# Wrong: unescaped special characters
--token-pattern "[INIT]"

# Correct: escape brackets
--token-pattern "\[INIT\]"

# Test pattern online: https://regex101.com/
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