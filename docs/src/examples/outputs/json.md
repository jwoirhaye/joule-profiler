# JSON Output Format

When using Joule Profiler with `--json`, results are exported as **structured JSON**.  
This format is suitable for programmatic processing, analysis, or exporting to other tools.

## Fields

A JSON output of Joule Profiler contains these fields:

| Field | Description |
|-------|-------------|
| `command` | Program and arguments executed |
| `token_pattern` | Regex used to detect phase tokens |
| `exit_code` | Return code of the program |
| `phases` | Array of measured phases |
| `index` | Phase index |
| `start_token` / `end_token` | Phase markers |
| `timestamp` | Start timestamp |
| `duration_ms` | Phase duration in milliseconds |
| `metrics` | List of metrics collected by sources |
| `metrics[].name` | Sensor name |
| `metrics[].value` | Measured value |
| `metrics[].unit` | Unit of measurement (e.g., µJ) |
| `metrics[].source` | Metric source name (e.g., `RAPL`, `NVML`) |

## Profile Example

The command:
```
joule-profiler --json profile -- python3 nbody.py 500000
```

Displays:

```json
{
  "command": "python3 nbody.py 500000",
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

## Listing Sensors

Here is an example of sensors listing in json format:

```
joule-profiler --json list-sensors
```

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