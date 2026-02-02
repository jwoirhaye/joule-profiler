# Output Formats

Joule Profiler supports several output formats to accommodate different usage scenarios and workflows. The **terminal format** is the default and is intended for quick inspection and human-readable feedback. It displays structured results directly in the console, including the command executed, phase durations, and per-source metrics. This format is ideal when you want to observe the results interactively or debug a profiling run. For details, see [Terminal Output Format](terminal.md). For now, the output formats are only available using the CLI.

For automated processing, further analysis, or integration with pipelines, Joule Profiler can export results in **JSON format** using the `--json` CLI flag. JSON provides a structured representation of the profiling session, including command information, phase metadata, metric values, and iteration details when applicable. This format is well-suited for programmatic consumption, logging, or exporting to other tools. See [JSON Output Format](json.md) for more information.

When metrics need to be analyzed in spreadsheet software, notebooks or imported into other tabular tools, the profiler can generate a **CSV output** via the `--csv` flag. Each metric is represented as a row with associated phase, iteration, and source information. This flattened format facilitates aggregation, plotting, or other analyses where a row-per-metric structure is advantageous. More details can be found in [CSV Output Format](csv.md).

Overall, the choice of output format depends on your workflow: the terminal is optimized for immediate human inspection, JSON is designed for programmatic workflows, and CSV is suited for tabular analyses and external data processing.

In the future, the output formats might be moved from the CLI module to another module, enabling users to use them outside of the CLI when they're using the library.