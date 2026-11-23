#!/usr/bin/env bash
# Shared setup for every start.sh step script. Source it, don't execute it.

BABYDRADE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export BABYDRADE_ROOT
cd "$BABYDRADE_ROOT"

# Ensure cargo is available in PATH when installed via rustup.
if [ -n "${HOME:-}" ] && [ -f "$HOME/.cargo/env" ]; then
    # shellcheck source=/dev/null
    . "$HOME/.cargo/env"
fi

# Console and child-process locale.
export LANG="en_US.UTF-8"
export LC_ALL="en_US.UTF-8"
export LANGUAGE="en_US:en"

# Every script sees the same runtime directory, even when the inherited one is
# missing or not writable.
if [ -z "${XDG_RUNTIME_DIR:-}" ] || [ ! -d "$XDG_RUNTIME_DIR" ] || [ ! -w "$XDG_RUNTIME_DIR" ]; then
    CURRENT_UID="$(id -u 2>/dev/null || echo 1000)"
    export XDG_RUNTIME_DIR="/tmp/babydrade-runtime-$CURRENT_UID"
    mkdir -p "$XDG_RUNTIME_DIR"
    chmod 0700 "$XDG_RUNTIME_DIR"
fi

if [ -z "${RUST_LOG:-}" ]; then
    export RUST_LOG="info,window_manager=debug,services=debug,babydrade=debug"
fi

USER_BIN_DIR="${HOME:-}/.local/bin"
SYSTEM_BIN_DIRS=("/usr/local/bin" "/usr/bin")
export USER_BIN_DIR

run_privileged() {
    if [ "$(id -u)" -eq 0 ]; then
        "$@"
    elif command -v sudo >/dev/null 2>&1; then
        sudo "$@"
    elif command -v doas >/dev/null 2>&1; then
        doas "$@"
    else
        echo "[!] Root privileges required to execute command, but neither 'sudo' nor 'doas' was found." >&2
        echo "[!] Please run: $*" >&2
        return 1
    fi
}

step() {
    echo "[*] $*"
}
