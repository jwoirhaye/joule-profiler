# Phase mode

The main feature that distinguishes **Joule Profiler** from other profilers such as Alumet[^alumet] or Scaphandre[^scaphandre] is its **phase mode**.
Phase mode enables energy profiling on different parts of a program, called phases, allowing to identify which sections of execution contribute most to energy consumption.

Phases are detected through tokens printed to standard output, matched using a configurable regular expression.
This approach may introduce overhead and noise depending on the system’s I/O performance.

In the future, an alternative approach may be implemented using inter-process communication with language-specific wrappers to minimize overhead.

[^alumet]: [Alumet](https://alumet.dev)
[^scaphandre]: [Scaphandre](https://github.com/hubblo-org/scaphandre)