# Introduction

**Joule Profiler** is a tool for measuring program metrics on Linux systems, with a focus on energy consumption.  
It supports profiling CPU, DRAM and other system metrics at different scopes and is designed for low-overhead measurement using asynchronous scheduling and a modular architecture.

The profiler is available today on Intel processors via the RAPL powercap framework.  
Support for additional platforms (AMD, ARM, GPU, etc.) will be added in the future through extensible metric sources.  
Some traits are exposed through the crate API to allow users to implement custom metric sources easily.

**Joule Profiler** is heavily inspired by JouleIt[^jouleit], but provides enhanced features and is written in Rust for better performance, safety, and portability.

## Phase mode

The main feature that distinguishes **Joule Profiler** from other profilers such as Alumet[^alumet] or Scaphandre[^scaphandre] is its **phase mode**.  
Phase mode enables energy profiling on different parts of a program, called phases, allowing developers to identify which sections contribute most to energy consumption.

Phases are detected through tokens printed to standard output, matched using a configurable regular expression.  
This approach may introduce overhead and noise depending on the system’s I/O performance.

In the future, an alternative approach may be implemented using inter-process communication with language-specific wrappers to minimize overhead.

[^jouleit]: [Jouleit](github.com/powerapi-ng/jouleit)  
[^alumet]: [Alumet](https://alumet.dev)  
[^scaphandre]: [Scaphandre](https://github.com/hubblo-org/scaphandre)