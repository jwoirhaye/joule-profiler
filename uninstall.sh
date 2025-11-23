#!/usr/bin/env bash
#
# joule-profiler uninstaller
#
set -e

readonly BINARY_NAME="joule-profiler"
readonly INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"
readonly COLOR_RESET='\033[0m'
readonly COLOR_GREEN='\033[0;32m'
readonly COLOR_RED='\033[0;31m'
readonly COLOR_BLUE='\033[0;34m'
readonly COLOR_YELLOW='\033[0;33m'

log_info() {
    echo -e "${COLOR_BLUE}ℹ${COLOR_RESET} $*" >&2
}

log_success() {
    echo -e "${COLOR_GREEN}✓${COLOR_RESET} $*" >&2
}

log_error() {
    echo -e "${COLOR_RED}✗${COLOR_RESET} $*" >&2
}

log_warning() {
    echo -e "${COLOR_YELLOW}⚠${COLOR_RESET} $*" >&2
}

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

main() {
    echo "" >&2
    echo "╔══════════════════════════════════════════╗" >&2
    echo "║   joule-profiler uninstaller             ║" >&2
    echo "╚══════════════════════════════════════════╝" >&2
    echo "" >&2

    local binary_path="$INSTALL_DIR/$BINARY_NAME"

    # Check if installed
    if [ ! -f "$binary_path" ]; then
        # Maybe it's installed elsewhere
        if command_exists "$BINARY_NAME"; then
            binary_path=$(command -v "$BINARY_NAME")
            log_warning "$BINARY_NAME found at $binary_path (not in $INSTALL_DIR)"
            log_info "This script will remove $binary_path"
        else
            log_error "$BINARY_NAME is not installed"
            exit 1
        fi
    fi

    log_info "Found $BINARY_NAME at $binary_path"

    # Get version before removing
    local version
    if command_exists "$BINARY_NAME"; then
        version=$("$BINARY_NAME" --version 2>/dev/null | grep -oP '\d+\.\d+\.\d+' || echo "unknown")
        log_info "Version: $version"
    fi

    # Confirm uninstallation
    echo -n "Do you want to uninstall $BINARY_NAME? [y/N] " >&2
    read -r reply
    if [[ ! $reply =~ ^[Yy]$ ]]; then
        log_info "Uninstallation cancelled"
        exit 0
    fi

    log_info "Removing $BINARY_NAME..."

    # Remove binary
    local install_dir
    install_dir=$(dirname "$binary_path")

    if [ -w "$install_dir" ]; then
        rm -f "$binary_path"
    else
        if ! command_exists sudo; then
            log_error "sudo is required to remove $binary_path"
            exit 1
        fi
        sudo rm -f "$binary_path"
    fi

    log_success "$BINARY_NAME has been uninstalled"
    echo "" >&2
}

main "$@"