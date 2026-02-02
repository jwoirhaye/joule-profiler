# Perf Events

## Overview

For a detailed overview of what **perf_events** is and how it works, see [perf_events](../perf_events.md), here we will discuss only about the measurements of RAPL domains counters.

To measure the energy consumption of the RAPL domains through **perf_events**, we're using the **perf_event_open_sys** rust crate, which is a wrapper around the **perf_event_open** Linux system call, used to open perf event counters. The crate also provides bindings to associated perf_events I/O controllers to manage the opened counters.

## Limitations

- We cannot obtain domains per-process energy consumption because