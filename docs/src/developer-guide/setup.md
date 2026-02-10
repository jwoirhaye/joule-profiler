# Development Setup

This section explains how to set up a local development environment for **Joule Profiler**, build the project, run tests, and contribute effectively.

## 1. Prerequisites

Before starting, ensure you have the following installed:

### System Requirements
- **Linux OS** (Ubuntu, Debian, Fedora, Arch, etc.)
- Intel CPU with **RAPL support** (most CPUs since Sandy Bridge, 2011)
- Root access (required to read RAPL counters with Powercap backend)

### Software Dependencies

- **rustc** 1.70+ and **cargo** (official Rust installer recommended)
- **mdBook** >= 0.4.40 and < 0.5 and its dependencies (for building documentation, see documentation requirements)

## 2. Clone the Repository

Use Git to clone the repository:

```bash
git clone https://github.com/jwoirhaye/joule-profiler.git
cd joule-profiler
```

## 3. Build the Project

Build in **release mode**:

```bash
cargo build --release
```

This will produce the binary at:

```
target/release/joule-profiler
```

You can run it directly using:

```bash
./target/release/joule-profiler --version
```

For **debug mode** (faster iteration during development):

```bash
cargo build
./target/debug/joule-profiler --version
```

## 4. Run Tests

Run the full test suite with:

```bash
cargo test
```

For faster iteration during development:

```bash
cargo test --lib        # only library tests
cargo test --bins       # only binary/tests for CLI
cargo test --doc       # only doctests
cargo test -- --nocapture  # show test output
```

## 5. Formatting & Linting

Ensure code follows project style using **rustfmt** and **clippy**:

```bash
cargo fmt       # format code
cargo clippy    # lint code for warnings and suggestions
```

> [!NOTE]
> Recommended to run these before committing changes and required for submitting pull requests.

## 6. Development Tips

- Use the `examples/` folder to experiment with different scripts and phase token patterns.
- Use logging flags (`-v`, `-vv`, `-vvv`) for debugging purposes.

## 7. Notes

> [!NOTE]
> - Energy measurements require **Intel CPUs** and Linux with the **powercap** framework or **perf_event** kernel module.
> - Running programs under a virtual machine may give inaccurate readings due to limited access to RAPL counters.
> - Root access is required for RAPL counters, but you can test some library functionality without root.

By following these steps, you’ll have a fully functional development environment for **Joule Profiler** and be ready to contribute or test new features.
