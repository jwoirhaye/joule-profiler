# CSV Output Format

Joule Profiler can also export results as **CSV** (semicolon-separated by default).  
This format is suitable for spreadsheets, scripting, or import into analysis tools.

---

## CSV Header

The CSV file contains the following columns:

| Column | Description |
|--------|-------------|
| `phase_id` | Phase index |
| `phase_name` | Phase name, e.g., `"START -> END"` |
| `phase_duration_ms` | Duration of the phase in milliseconds |
| `metric_name` | Name of the metric (sensor) |
| `metric_value` | Measured value |
| `metric_unit` | Unit of the measurement (e.g., µJ) |
| `metric_source` | Source of the metric (e.g., `powercap`) |
| `start_token` | Phase start token |
| `end_token` | Phase end token |
| `start_line` | Line number in program output where phase started (optional) |
| `end_line` | Line number in program output where phase ended (optional) |
| `timestamp` | Start timestamp of the phase (ms since epoch) |
| `command` | Command executed |
| `exit_code` | Program exit code |
| `token_pattern` | Regex used to detect phase tokens |

## Single Iteration

```csv
phase_id;phase_name;phase_duration_ms;metric_name;metric_value;metric_unit;metric_source;start_token;end_token;start_line;end_line;timestamp;command;exit_code;token_pattern
0;"START -> END";1859;CORE-0;45935552;µJ;powercap;START;END;;;1769987854341;"python3 nbody.py 500000";0;"__[A-Z0-9_]+__"
0;"START -> END";1859;DRAM-0;1283811;µJ;powercap;START;END;;;1769987854341;"python3 nbody.py 500000";0;"__[A-Z0-9_]+__"
0;"START -> END";1859;PACKAGE-0;66560987;µJ;powercap;START;END;;;1769987854341;"python3 nbody.py 500000";0;"__[A-Z0-9_]+__"
```

Which gives:

| phase_id | phase_name    | phase_duration_ms | metric_name | metric_value | metric_unit | metric_source | start_token | end_token | start_line | end_line | timestamp       | command                    | exit_code | token_pattern      |
|----------|---------------|-----------------|-------------|--------------|-------------|---------------|-------------|-----------|------------|----------|----------------|----------------------------|-----------|------------------|
| 0        | START -> END  | 1859            | CORE-0      | 45935552     | µJ          | powercap      | START       | END       |            |          | 1769987854341  | python3 nbody.py 500000   | 0         | __[A-Z0-9_]+__   |
| 0        | START -> END  | 1859            | DRAM-0      | 1283811      | µJ          | powercap      | START       | END       |            |          | 1769987854341  | python3 nbody.py 500000   | 0         | __[A-Z0-9_]+__   |
| 0        | START -> END  | 1859            | PACKAGE-0   | 66560987     | µJ          | powercap      | START       | END       |            |          | 1769987854341  | python3 nbody.py 500000   | 0         | __[A-Z0-9_]+__   |


- This minimal CSV corresponds to a **single run** of the program.
- Each metric is on a separate row, even if coming from the same phase.
- Useful for quick analysis or import into spreadsheet tools.

## Multiple Iterations

The header with multiple iterations includes the iteration index at first column:

```csv
iteration_id;phase_id;phase_name;phase_duration_ms;metric_name;metric_value;metric_unit;metric_source;start_token;end_token;start_line;end_line;timestamp;command;exit_code;token_pattern
0;0;"START -> END";1871;CORE-0;43888865;µJ;powercap;START;END;;;1769987846811;"python3 nbody.py 500000";0;"__[A-Z0-9_]+__"
0;0;"START -> END";1871;DRAM-0;1291012;µJ;powercap;START;END;;;1769987846811;"python3 nbody.py 500000";0;"__[A-Z0-9_]+__"
0;0;"START -> END";1871;PACKAGE-0;64663103;µJ;powercap;START;END;;;1769987846811;"python3 nbody.py 500000";0;"__[A-Z0-9_]+__"
1;0;"START -> END";1858;CORE-0;43495189;µJ;powercap;START;END;;;1769987848682;"python3 nbody.py 500000";0;"__[A-Z0-9_]+__"
1;0;"START -> END";1858;DRAM-0;1281125;µJ;powercap;START;END;;;1769987848682;"python3 nbody.py 500000";0;"__[A-Z0-9_]+__"
1;0;"START -> END";1858;PACKAGE-0;64109211;µJ;powercap;START;END;;;1769987848682;"python3 nbody.py 500000";0;"__[A-Z0-9_]+__"
```

Which gives:

| iteration_id | phase_id | phase_name    | phase_duration_ms | metric_name | metric_value | metric_unit | metric_source | start_token | end_token | start_line | end_line | timestamp       | command                    | exit_code | token_pattern      |
|--------------|----------|---------------|-----------------|-------------|--------------|-------------|---------------|-------------|-----------|------------|----------|----------------|----------------------------|-----------|------------------|
| 0            | 0        | START -> END  | 1871            | CORE-0      | 43888865     | µJ          | powercap      | START       | END       |            |          | 1769987846811  | python3 nbody.py 500000   | 0         | __[A-Z0-9_]+__   |
| 0            | 0        | START -> END  | 1871            | DRAM-0      | 1291012      | µJ          | powercap      | START       | END       |            |          | 1769987846811  | python3 nbody.py 500000   | 0         | __[A-Z0-9_]+__   |
| 0            | 0        | START -> END  | 1871            | PACKAGE-0   | 64663103     | µJ          | powercap      | START       | END       |            |          | 1769987846811  | python3 nbody.py 500000   | 0         | __[A-Z0-9_]+__   |
| 1            | 0        | START -> END  | 1858            | CORE-0      | 43495189     | µJ          | powercap      | START       | END       |            |          | 1769987848682  | python3 nbody.py 500000   | 0         | __[A-Z0-9_]+__   |
| 1            | 0        | START -> END  | 1858            | DRAM-0      | 1281125      | µJ          | powercap      | START       | END       |            |          | 1769987848682  | python3 nbody.py 500000   | 0         | __[A-Z0-9_]+__   |
| 1            | 0        | START -> END  | 1858            | PACKAGE-0   | 64109211     | µJ          | powercap      | START       | END       |            |          | 1769987848682  | python3 nbody.py 500000   | 0         | __[A-Z0-9_]+__   |

**Notes on this example:**

- Each **iteration** repeats the phase and metric rows.
- Multiple metrics from different sources (or sensors) are included as separate rows.
- `iteration_id` increments per program run; `phase_id` increments per detected phase.
- Empty `start_line` / `end_line` indicate that line tracking is not enabled.

## Listing Sensors

Here is an example of sensors listing in CSV format:

```
sensor;unit;source
PSYS-1;µJ;powercap
PACKAGE-0;µJ;powercap
CORE-0;µJ;powercap
UNCORE-0;µJ;powercap
```

which gives the following table:

| sensor | unit | source |
|--------------|----------|---------------|
| PSYS-1 | µJ | powercap |
| PACKAGE-0 | µJ | powercap |
| CORE-0 | µJ | powercap |
| UNCORE-0 | µJ | powercap |

## Notes

- CSV is **semicolon-separated** for compatibility with most spreadsheet software.
- Each row represents **one metric** for a single phase.
- Multiple iterations are flattened in sequence, making it easy to filter by `iteration_id`.
- Programmatic tools can aggregate phases, iterations, and sources easily using the CSV columns.
