# Introduction

`Joule Profiler` is a tool for measuring programs metrics on Linux systems.
It measures programs energy consumption and other metrics of CPU, DRAM and more at different scopes.

For now, **Joule Profiler** uses only Intel RAPL with powercap framework to leverage programs energy consumption.

It is heavily inspired by [JouleIt](github.com/powerapi-ng/jouleit), but with enhanced features and written in Rust for better performance.