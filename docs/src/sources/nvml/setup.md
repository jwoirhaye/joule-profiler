# Setup & Requirements

To use the NVML source in **Joule Profiler**, your system must meet specific hardware and software requirements.

## Operating System Support

The **NVML** source is cross-platform and supported on **Linux** and **Windows**.

## Driver Requirements

Joule Profiler accesses NVML via the shared library `libnvidia-ml.so` (usually in `/usr/lib/` or `/usr/lib64/` on Linux) installed alongside the NVIDIA drivers. You do not need to install the CUDA toolkit manually, but the base GPU drivers are required.

> [!NOTE]
> These libraries are installed automatically with standard NVIDIA Display Drivers.

## Hardware Support

Energy consumption metrics are available on NVIDIA GPUs based on the **Volta architecture** (e.g., Tesla V100, Titan V) and newer. This includes all modern consumer architectures starting from **Turing** (RTX 20 series). Older architectures (Pascal, Maxwell, Kepler) may report other metrics but do not expose the energy counters required by this profiler.

## Verification

Before running **Joule Profiler**, you can verify that your drivers are correctly installed and that your GPU supports management queries using the standard `nvidia-smi` tool.

Run the following command in your terminal:

```bash
nvidia-smi -q -d POWER
```

The output should look similar to the following:

```
==============NVSMI LOG==============

Timestamp                                 : Sat Feb  7 11:17:58 2026
Driver Version                            : 550.163.01
CUDA Version                              : 12.4

Attached GPUs                             : 1
GPU 00000000:01:00.0
    GPU Power Readings
        Average Power Draw                : N/A
        Instantaneous Power Draw          : 31.64 W
        Current Power Limit               : 140.00 W
        ...
```

> [!WARNING]
> If your output does not show "Power Draw" or "Energy Consumption," your hardware may not support the necessary features for this profiler.