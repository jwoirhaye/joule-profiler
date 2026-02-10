# Overview

**Joule Profiler** is designed to minimize measurement overhead while maintaining high performance, modularity, extensibility, and a strong separation of concerns.

This architecture enables:

- Efficient asynchronous scheduling.
- Low-overhead metric collection.
- Easy integration of new metric sources.
- User-defined metric source extensions without modifying the core.

```mermaid
flowchart LR
%%{init: {'flowchart': {'nodeSpacing': 20, 'rankSpacing': 30}}}%%

subgraph Sources
    RAPL
    NVML(Nvidia-Nvml)
    PERF(Perf events)
end

JouleProfiler((JouleProfiler))
Orchestrator

subgraph OUTPUTS[Output formats]
    Terminal
    JSON
    CSV
end

JouleProfiler e1@-->|Measure phases| Orchestrator
e1@{ animate: true }

JouleProfiler -->|Retrieve event| Orchestrator
Orchestrator -->|Join + retrieve results| JouleProfiler

Orchestrator e2@-->|Schedule| Sources
e2@{ animate: fast }

Orchestrator -->|Join tasks| Sources
Orchestrator e3@-->|Measure events| Sources
e3@{ animate: true }

JouleProfiler --> Terminal
JouleProfiler --> JSON
JouleProfiler --> CSV

linkStyle 0 stroke:#e67e22, stroke-width:2px
linkStyle 1 stroke:#e74c3c, stroke-width:2px
linkStyle 2 stroke:#e74c3c, stroke-width:2px
linkStyle 3 stroke:#9b59b6, stroke-width:2px
linkStyle 4 stroke:#e74c3c, stroke-width:2px
linkStyle 5 stroke:#e67e22, stroke-width:2px
linkStyle 6 stroke:#27ae60, stroke-width:2px
linkStyle 7 stroke:#27ae60, stroke-width:2px
linkStyle 8 stroke:#27ae60, stroke-width:2px
```

## High-Level Design

At a high level, Joule Profiler is composed of three main layers:

- [**Core Module**](core-module.md) – Contains all domain logic: orchestration, aggregation, and result modeling.
- [**CLI Module**](cli-module.md) – Responsible for user input, command line arguments parsing, and startup wiring.
- [**Sources Workspace**](sources-workspace.md) – Implementations of the different metric sources using the API traits.