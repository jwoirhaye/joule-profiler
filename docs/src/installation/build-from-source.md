# Build from Source

You can build the profiler from the sources by cloning the repository. It can be useful if you want to access the latest features not yet released, or if you intend to customize the source code.

```bash
# Clone the repository
git clone https://github.com/jwoirhaye/joule-profiler.git
cd joule-profiler

# Build and install system-wide
cargo build --release
sudo cp target/release/joule-profiler /usr/local/bin/

# Verify installation
sudo joule-profiler --version
```

> [!TIP]
> System-wide installation (`/usr/local/bin/`) is recommended as the tool requires `sudo` to access RAPL counters.

