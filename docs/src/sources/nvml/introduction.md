# NVIDIA Management Library (NVML)

## Overview

The **NVIDIA Management Library (NVML)** is a C-based programmatic interface for monitoring and managing NVIDIA GPUs. It is the underlying library used by the standard **nvidia-smi**[^nvidia-smi] tool and provides direct access to the GPU driver.

**Joule Profiler** interfaces with NVML using the **nvml-wrapper**[^nvml-wrapper] Rust crate to retrieve reliable hardware energy counters.

## Architecture

NVML is distributed as `libnvidia-ml.so` on Linux or `nvml.dll` on Windows (not supported) and communicates directly with the NVIDIA GPU driver to query device state.

<div style="text-align:center"><img src="../../figures/nvml_architecture.png" /></div>

[^nvidia-smi]: [System Management Interface SMI (nvidia-smi)](https://developer.nvidia.com/system-management-interface)
[^nvml-wrapper]: [nvml-wrapper](https://github.com/rust-nvml/nvml-wrapper)