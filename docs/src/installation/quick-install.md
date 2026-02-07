# Quick Install

Install the latest version with a single command:

```bash
curl -fsSL https://raw.githubusercontent.com/jwoirhaye/joule-profiler/main/install.sh | bash
```

This script automatically detects your operating system and architecture, downloads the correct pre-built binary, and installs it to `/usr/local/bin/`.

> [!NOTE]
> You may be prompted for your sudo password during installation to move the binary to the system path.

# Custom Installation

For more control over the installation process, you can pass arguments to the installation script.

**Install to a custom directory**, useful for non-root users or specific environment configurations (e.g., `~/.local/bin`):

```bash
curl -fsSL https://raw.githubusercontent.com/jwoirhaye/joule-profiler/main/install.sh | bash -s -- --dir ~/.local/bin
```

**Install specific version:**

```bash
curl -fsSL https://raw.githubusercontent.com/jwoirhaye/joule-profiler/main/install.sh | bash -s -- --version v0.1.0
```

**List available versions:**

```bash
curl -fsSL https://raw.githubusercontent.com/jwoirhaye/joule-profiler/main/install.sh | bash -s -- --list
```

**Non-interactive Mode**, for use in automated scripts or CI pipelines:

```bash
curl -fsSL https://raw.githubusercontent.com/jwoirhaye/joule-profiler/main/install.sh | bash -s -- --yes
```