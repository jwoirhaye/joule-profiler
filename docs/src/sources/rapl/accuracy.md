# Measurement Accuracy

**Joule Profiler** relies on the MSR (Model-Specific Register) counters provided by the **RAPL** interface to deliver CPU energy metrics at different scopes.

## Measurement Unit

**RAPL** energy counters report measurements in **microjoules ($\mu\text{J}$)**. This high-precision unit ($10^{-6}$ joules) theoretically allows for extremely detailed energy accounting. However, it is important to note that while the data format is precise, the effective accuracy is bound by the hardware's update frequency and the specific implementation of the voltage regulators on the motherboard.

# Limitations

While **RAPL** is a powerful tool, it has inherent constraints that users should be aware of when interpreting results.

## Lack of Per-Process Attribution

Although the **RAPL** interface provides multiple domains for fine-grained energy profiling, measurements are performed at the hardware level. Thus, **it does not natively support per-process energy attribution**. This makes it difficult to accurately assess the isolated energy consumption of a single process.

## Hardware Variability

The availability of specific power domains is strictly hardware-dependent. Domains such as **DRAM** or **PSYS** (Platform System) might not be available depending on the specific CPU generation or platform configuration. Energy measurements can also be noisier on some systems, such as laptops, due to more aggressive power management and variable clock speeds.

## Temporal Resolution

Very short or highly variable workloads may not be measured accurately. The hardware counters update at a fixed rate, so rapid changes in energy consumption can be missed between samples. For this reason, longer-running workloads generally produce more reliable results than microbenchmarks.
