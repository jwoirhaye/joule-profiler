# Sources Workspace

The Sources Workspace contains all metric source implementations (e.g., [RAPL](../sources/rapl/introduction.md), [perf_event](../sources/perf_event/introduction.md), [NVML](../sources/nvml/introduction.md), etc.).
Each source is responsible for collecting a specific type of measurement data.

Metric sources are responsible for collecting raw measurement data and maintaining any internal state required for measurement. Sources never perform aggregation logic but provide functions to transform raw data to aggregated metrics at the end of the profiling.

Each source is isolated and independent from other sources and can be configured independently.
This allows users to enable only the sources they need and makes it easy to add new ones.

The workspace is designed to support various sources, including built-in sources maintained by the project or user-defined sources.
New sources can be added without modifying the core or the CLI, as long as they follow the expected interface.