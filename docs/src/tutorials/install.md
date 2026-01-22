# Installation

## Quick Install

Install the latest version with a single command:

```bash
curl -fsSL https://raw.githubusercontent.com/jwoirhaye/joule-profiler/main/install.sh | bash
```

## Custom Installation

Install to custom directory:
```bash
curl -fsSL https://raw.githubusercontent.com/jwoirhaye/joule-profiler/main/install.sh | bash -s -- --dir ~/.local/bin
```

Install specific version:
```bash
curl -fsSL https://raw.githubusercontent.com/jwoirhaye/joule-profiler/main/install.sh | bash -s -- --version v0.1.0
```

List available versions:
```bash
curl -fsSL https://raw.githubusercontent.com/jwoirhaye/joule-profiler/main/install.sh | bash -s -- --list
```
Non-interactive (for CI/CD):
```bash
curl -fsSL https://raw.githubusercontent.com/jwoirhaye/joule-profiler/main/install.sh | bash -s -- --yes
```

---

## From Source

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

**Note:** System-wide installation (`/usr/local/bin/`) is recommended as the tool requires `sudo` to access RAPL counters.

---

## Uninstall

Using uninstaller:
```bash
curl -fsSL https://raw.githubusercontent.com/jwoirhaye/joule-profiler/main/uninstall.sh | bash
```

Or manually:
```bash
sudo rm /usr/local/bin/joule-profiler
```