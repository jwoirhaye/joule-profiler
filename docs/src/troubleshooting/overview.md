# Troubleshooting

## Overview

Energy profiling operates at a low level, directly interfacing with hardware counters and kernel subsystems. This introduces several challenges:

- **Hardware Dependencies**: Energy counters (RAPL, perf_event) require specific CPU support and may be disabled in BIOS
- **Privilege Requirements**: Reading hardware counters typically requires elevated permissions or specific kernel configurations
- **Measurement Sensitivity**: Energy readings are affected by system state, background processes, and thermal conditions
- **Platform Variations**: Different CPUs, kernel versions, and system configurations expose metrics differently

Most issues stem from these fundamental constraints rather than bugs in the profiler itself.

See [common-issues](common issues.md) if you encounter an issue with the profiler and do not find it in the common issues, then please submit it on the GitHub repository and we will work on a solution.