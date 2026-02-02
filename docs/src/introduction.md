# Introduction

**Joule Profiler** is a tool for measuring program metrics on Linux systems, with a focus on energy consumption.
It supports profiling CPU, DRAM and other system metrics at different scopes and is designed for low-overhead measurement. Its flexible hexagonal architecture provides modularity and extensibility that allows the implementation of new sources easily and its asynchronous runtime with tokio minimizes the introduced overhead to provide the most accurate measures.

It is usable through the CLI (see getting started), or through the exposed library, which provides more flexibility in configuration and the possibility to add user-defined sources.

The profiler is available today on Intel processors via the RAPL powercap framework.
Support for additional platforms (AMD, ARM, GPU, etc.) will be added in the future through extensible metric sources.
Some traits are exposed through the crate API to allow users to implement custom metric sources easily.

**Joule Profiler** is heavily inspired by JouleIt[^jouleit], but provides enhanced features and is written in Rust for better performance, safety, portability, and extensibility.

[^jouleit]: [Jouleit](github.com/powerapi-ng/jouleit)