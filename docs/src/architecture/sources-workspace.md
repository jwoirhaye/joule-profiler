# Sources Workspace

The Sources Workspace contains all metric source implementations.

Each source is responsible for collecting a specific type of measurement data.

## Usage

Metric sources are responsible for:
- Interfacing with external systems or measurement providers
- Collecting raw measurement data
- Maintaining any internal state required for measurement
- Returning finalized results to the core

Sources never perform aggregation or reporting logic but provide functions to transform raw data to aggregated metrics.

## Independence

Each source is:
- Isolated and independent from other sources
- Configured independently
- Replaceable without affecting the core

This allows users to enable only the sources they need and makes it easy to add new ones.

## Extensibility

The workspace is designed to support:
- Built-in sources maintained by the project
- Experimental sources
- User-defined or third-party sources

New sources can be added without modifying the core or the CLI, as long as they follow the expected interface.

Sources are designed to be reusable across multiple profiling sessions.