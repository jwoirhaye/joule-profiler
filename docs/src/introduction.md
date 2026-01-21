# Introduction

**Joule Profiler** is a tool for measuring programs metrics on Linux systems.
It measures programs energy consumption and other metrics of CPU, DRAM and more at different scopes.

For now it is available on Intel processors, but in the future we will add various sources to profile GPU, AMD and ARM processors support.
Some traits are accessible through the crate API to able users to implement their own metrics sources easily. 

For now, **Joule Profiler** uses only Intel RAPL with powercap framework to leverage programs energy consumption.

It is heavily inspired by JouleIt[^jouleit], but with enhanced features and written in Rust for better performance.

## Phase mode

The main goal of **Joule Profiler** and the thing that differenciate it from other profilers like Alumet[^alumet] or Scaphandre[^scaphandre] is that it implements
a phase mode which is energy profiling on different parts of the program called phases. It allows to study and find what parts of a program lead to more consumption.

Phases are detected through tokens printed in the standard output, matching a configurable regular expression, this approach can introduce overhead and noise in the results
depending on the system's I/O performance.

In the future, another approach may be implemented using inter-process communication with wrappers in multiple languages to minimize the introduced overhead.

[^jouleit]: [Jouleit](github.com/powerapi-ng/jouleit)
[^alumet]: [Alumet](https://alumet.dev)
[^scaphandre]: [Scaphandre](https://github.com/hubblo-org/scaphandre)