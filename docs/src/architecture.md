## Architecture

To assess energy consumption efficiently, we are using the powercap framework.
Our goal is to minimize the impact of our tool and measurements on the real energy consumption, we also try to minimize the warmup of the machine to avoid bias.

Firstly, we retrieve the RAPL domains through the powercap framework **sysfs**, it is possible to filter domains through CLI or configuration structure.

Then, we launch the worker thread responsible for RAPL measurements.
Before launching the benchmarked program, we make a first measure of reference.

In phases mode, we make a measure when a token matches the provided regular expression.

After the end of the program, we make the last measure to have the difference within each measures through all the lifecycle of the program.