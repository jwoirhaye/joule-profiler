# JSON Output Format

When using Joule Profiler with `--json`, results are exported as **structured JSON**.  
This format is suitable for programmatic processing, analysis, or exporting to other tools.

## Single Iteration / Single Phase

A minimal example looks like this:

```json
{
  "command": "python3 nbody.py 500000",
  "mode": "phases",
  "token_pattern": "__[A-Z0-9_]+__",
  "exit_code": 0,
  "phases": [
    {
      "index": 0,
      "start_token": "START",
      "end_token": "END",
      "timestamp": 1769987285805,
      "duration_ms": 1895,
      "metrics": [
        {
          "name": "CORE-0",
          "value": 45010261,
          "unit": "µJ",
          "source": "powercap"
        },
        {
          "name": "DRAM-0",
          "value": 1302425,
          "unit": "µJ",
          "source": "powercap"
        },
        {
          "name": "PACKAGE-0",
          "value": 51537160,
          "unit": "µJ",
          "source": "powercap"
        }
      ]
    }
  ]
}
```

**Fields explained:**

| Field | Description |
|-------|-------------|
| `command` | Program and arguments executed |
| `mode` | `"phases"` for single iteration, single run |
| `token_pattern` | Regex used to detect phase tokens |
| `exit_code` | Return code of the program |
| `phases` | Array of measured phases |
| `index` | Phase index |
| `start_token` / `end_token` | Phase markers |
| `timestamp` | Start timestamp (ms since epoch) |
| `duration_ms` | Phase duration in milliseconds |
| `metrics` | List of metrics collected by sources |
| `metrics[].name` | Sensor name |
| `metrics[].value` | Measured value |
| `metrics[].unit` | Unit of measurement (e.g., µJ) |
| `metrics[].source` | Metric source name (e.g., `powercap`) |

## Multiple Iterations

When profiling with multiple iterations, the JSON structure includes an `iterations` array:

```json
{
  "command": "python3 nbody.py 500000",
  "mode": "phases-iterations",
  "token_pattern": "__[A-Z0-9_]+__",
  "nb_iterations": 2,
  "iterations": [
    {
      "index": 0,
      "timestamp": 1769987356916,
      "duration_ms": 1901,
      "exit_code": 0,
      "phases": [
        {
          "index": 0,
          "start_token": "START",
          "end_token": "END",
          "timestamp": 1769987356916,
          "duration_ms": 1901,
          "metrics": [
            {"name": "CORE-0", "value": 45778752, "unit": "µJ", "source": "powercap"},
            {"name": "DRAM-0", "value": 1313351, "unit": "µJ", "source": "powercap"},
            {"name": "PACKAGE-0", "value": 66886364, "unit": "µJ", "source": "powercap"}
          ]
        }
      ]
    },
    {
      "index": 1,
      "timestamp": 1769987358817,
      "duration_ms": 1931,
      "exit_code": 0,
      "phases": [
        {
          "index": 0,
          "start_token": "START",
          "end_token": "END",
          "timestamp": 1769987358817,
          "duration_ms": 1931,
          "metrics": [
            {"name": "CORE-0", "value": 47500306, "unit": "µJ", "source": "powercap"},
            {"name": "DRAM-0", "value": 1320737, "unit": "µJ", "source": "powercap"},
            {"name": "PACKAGE-0", "value": 68928901, "unit": "µJ", "source": "powercap"}
          ]
        }
      ]
    }
  ]
}
```

**Additional fields for iterations:**

| Field | Description |
|-------|-------------|
| `nb_iterations` | Total number of iterations run |
| `iterations` | Array of each iteration's results |
| `iterations[].index` | Iteration index |
| `iterations[].timestamp` | Start timestamp of the iteration |
| `iterations[].duration_ms` | Duration of the iteration |
| `iterations[].exit_code` | Exit code of the program in this iteration |
| `iterations[].phases` | Phases measured within this iteration |

> Each iteration repeats the same phase structure, making it easy to compare metrics across iterations.

## Listing Sensors

Here is an example of sensors listing in json format:

```json
[
  {
    "name": "PSYS-1",
    "unit": "µJ",
    "source": "powercap"
  },
  {
    "name": "PACKAGE-0",
    "unit": "µJ",
    "source": "powercap"
  },
  {
    "name": "CORE-0",
    "unit": "µJ",
    "source": "powercap"
  },
  {
    "name": "UNCORE-0",
    "unit": "µJ",
    "source": "powercap"
  }
]
```