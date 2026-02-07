# Using the CLI

The **Joule Profiler CLI** provides a quick and flexible way to measure program energy consumption using various sources such as **RAPL**, **NVML**, or **Perf events**.  

The CLI is designed to be **ready-to-use** while also offering configuration options for advanced use cases.

## Basic Usage

```bash
sudo joule-profiler [OPTIONS] <SUBCOMMAND>
```

- Running with `sudo` may be required to access certain system metrics (e.g., RAPL counters).  
- `<SUBCOMMAND>` is either the profiling (phases) or sensors listing (list-sensors).

## Global Options

| Option | Description |
|--------|-------------|
| `-v`, `-vv`, `-vvv` | Increase verbosity of logging |
| `--rapl-path <PATH>` | Override the base path used to read RAPL counters (default: `/sys/devices/virtual/powercap/intel-rapl` or the environment variable `$JOULE_PROFILER_RAPL_PATH`) |
| `-s, --sockets <SOCKETS>` | Specify sockets to measure (e.g., `0` or `0,1`) |
| `--json` | Export results in JSON format |
| `--csv` | Export results in CSV format |
| `-o, --output-file <FILE>` | Output file for CSV / JSON results (default: `data<TIMESTAMP>.csv/json`) |

## Subcommands

### `phases`

Phase-based measurement mode, useful for measuring specific program phases identified by tokens in the program output.

**Options:**

| Option | Description |
|--------|-------------|
| `--token-pattern <REGEX>` | Regex pattern to detect phase tokens in program output (default: `__[A-Z0-9_]+__`) |
| `-n, --iterations <NUM>` | Number of iterations to run (default: 1) |
| `-o, --stdout-file <FILE>` | Redirect profiled program stdout to a file |
| `--rapl-polling <SECONDS>` | Set the polling frequency for RAPL measurements (e.g., `1` for 1s or `0.001` for 1ms) |
| `--` | Everything after `--` is treated as the command to execute |

**Example:**

```bash
sudo joule-profiler phases --rapl-polling 1 -- python3 nbody.py
```

- Runs the `nbody.py` benchmark.
- RAPL is polled every **1 second**.
- The profiler automatically detects phases using the default token pattern.

### `list-sensors`

List all available sensors that the profiler can access on the current system.

**Example:**

```bash
joule-profiler list-sensors
```

- Displays RAPL domains, GPU energy sensors, Perf events, etc.
- Useful to check what measurements are available before profiling.

> You should be able to list the available sensors without needing root rights.

## Output Formats

The profiler supports multiple output formats:

1. **Terminal (default)** – Pretty-printed results in the console.
2. **JSON** – Structured data for further processing.
3. **CSV** – Semicolon-separated values for spreadsheet or programmatic analysis.

> Use `--json` or `--csv` to select the output format, also they are mutually exclusive. You can also specify an output file with `-o`.

> [!NOTE]
> - Using `sudo` may be required to read certain counters (RAPL, perf counters).  
- Polling frequency affects precision and overhead: lower values may slightly increase CPU usage.