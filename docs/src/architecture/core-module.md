# Core Module

The Core Module is the engine of **Joule Profiler**, it defines the profiling model, coordinates measurements, and produces structured results.

The **Core Module** is responsible for the management and coordination of metric sources. It collects and aggregates results and exposes an easy to use API.
It contains no user interface logic and no hardware-specific code.

The core coordinates all metric sources through a central orchestrator. It sends events to manage the metric sources and make the measurements. At the end of the profiling, it collects all the data from sources and produces a unified result set.
Metric sources never interact with each other directly.
All coordination flows through the core and is hidden from the sources.

This design ensure that the implementation of new metric sources, output formats, or CLI features should not require changes to the core domain logic.
This ensures that existing workflows remain stable while the ecosystem grows.

During a profiling session, the core controls when measurements are made.
Measurements are associated with phase boundaries and accumulated across iterations.
All raw data is collected first, then processed once the measurements complete.
This separation ensures consistent results and reduce the measurements overhead.
