# CLI Module

The CLI Module is the straightforward entry point for users.

It parses command-line input and configure the profiler to be usable quickly and easily while providing several configuration options.
The CLI acts as an adapter between the user, the core domain, and the sources.
Because of this separation, the CLI can evolve independently from the core logic and sources.

Here's a summary of all the CLI arguments:

| Argument | Short | Long | Default | Description |
|---|---|---|---|---|
| `verbose` | `-v` | `--verbose` | `0` | Verbosity level. Stack for more detail: `-v`, `-vv`, `-vvv`. |
| `rapl_path` || `--rapl-path` | Auto-detected | Override the base path for Intel RAPL counters. Falls back to `$JOULE_PROFILER_RAPL_PATH`, then `/sys/devices/virtual/powercap/intel-rapl`. |
| `sockets` | `-s` | `--sockets` | All | CPU sockets to measure (e.g., `0` or `0,1`). |
| `json` || `--json` | `false` | Export results as JSON. Conflicts with `--csv`. |
| `csv` || `--csv` | `false` | Export results as semicolon-separated CSV. Conflicts with `--json`. |
| `output_file` | `-o` | `--output-file` | `data<TIMESTAMP>.csv/json` | Custom output file path for CSV/JSON export. |
| `gpu` || `--gpu` | `false` | Enable GPU measurement support. |
| `perf` || `--perf` | `false` | Enable `perf_event` hardware counters. |
| `rapl_backend` || `--rapl-backend` | `Perf` | Choose RAPL backend: `powercap` or `perf`. |
| `command` || *(subcommand)* | *(required)* | The subcommand/program to execute and profile. |

The subcommand `profile` has also some arguments:

| Argument | Short | Long | Default | Description |
|---|---|---|---|---|
| `token_pattern` || `--token-pattern` | `__[A-Z0-9_]+__` | Regex to detect phase tokens in the program's stdout. If the pattern contains a capture group, the captured text becomes the token name. Phases are computed between consecutive tokens (and from START/END). |
| `stdout_file` | `-o` | `--stdout-file` | None | Redirect the profiled program's stdout to a file. |
| `rapl_polling` || `--rapl-polling` | None | RAPL counter polling frequency, in seconds. |

The profiled command is provided after `--` at the end of the arguments.