# perf_event

## Overview

For a detailed overview of what **perf_event** is and how it works, see [perf_event](../perf_event.md), here we will discuss only about the measurements of RAPL domains counters.

To measure the energy consumption of the RAPL domains through perf_event, we're using the **perf_event_open_sys** rust crate, which is a wrapper around the **perf_event_open** Linux system call, used to open perf event counters. The crate also provides bindings to associated perf_event I/O controllers to manage the opened counters.

## Overflow handling ?

Unlike [**powercap**](powercap.md), perf_event handles the MSRs overflows and store counters on eight bytes. Moreover, the counters start at zero, which make them easy to compute and also completely prevent overflows, for example, on a CPU consuming at an average of 200 W, it would take 2924 years to overflow the **package** domain.

$$ P = 200~\text{W} = 200~\text{J/s} = 2.0 \times 10^8~\mu\text{J/s} $$

$$ t_\text{overflow} = \frac{E_\text{max}}{P} = \frac{2^{64}~\mu\text{J}}{2.0 \times 10^8~\mu\text{J/s}} \simeq 9.2 \times 10^{10}~\text{s} \simeq 2924~\text{years} $$

Thus, using perf_event, we do not need to implement polling strategies like with powercap, it is impossible for a domain counter to overflow.

## Limitations

- Unlike most perf_event counters, per-process energy consumption cannot be inferred for RAPL domains. This is because RAPL is not designed to expose per-process energy measurements, and perf_event does not perform aggregation of RAPL counters to estimate per-process domain-level energy consumption.
- RAPL counters exposed through perf do not support event grouping, which prevents managing all RAPL domains via a single ioctl and necessitates independent management of each domain.