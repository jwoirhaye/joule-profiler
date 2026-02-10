# CLI Module

The CLI Module is the straightforward entry point for users.

It parses command-line input and configure the profiler to be usable quickly and easily while providing several configuration options.

## Responsibilities

The CLI Module is responsible for:
- Parsing command-line arguments
- Validating user input
- Initializing and configurating sources
- Displaying or exporting results in various formats
- Configurating the profiler and launching it

It does not perform measurements or calculations itself.
The CLI acts as an adapter between the user and the core domain.

Because of this separation, the CLI can evolve independently from the core logic.