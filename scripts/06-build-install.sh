#!/usr/bin/env bash
# 6. Build the workspace and install the binaries into system bin directories.
set -eu
. "$(dirname "$0")/common.sh"

BINARIES=(
    babydra-window-manager
    babydra-desktop
    babydra-topbar
    babydra-greeter
)

step "Building BabyDraDE binaries (release)..."
cargo build --release --workspace

step "Installing BabyDraDE binaries into system directories..."
mkdir -p "$USER_BIN_DIR"

for bin in "${BINARIES[@]}"; do
    src="$BABYDRADE_ROOT/target/release/$bin"
    if [ ! -f "$src" ]; then
        echo "[!] Built binary not found: $src" >&2
        exit 1
    fi

    installed=0
    for sdir in "${SYSTEM_BIN_DIRS[@]}"; do
        [ -d "$sdir" ] || continue
        if [ -w "$sdir" ]; then
            if install -m 0755 "$src" "$sdir/$bin" 2>/dev/null; then
                installed=1
            fi
        elif run_privileged install -m 0755 "$src" "$sdir/$bin" 2>/dev/null; then
            installed=1
        fi
    done

    # Fallback for unprivileged sessions: still installed on PATH, never run
    # directly from target/ in the source tree.
    install -m 0755 "$src" "$USER_BIN_DIR/$bin"
    if [ "$installed" -eq 0 ]; then
        echo "[*] $bin installed to $USER_BIN_DIR (no writable system bin directory)."
    fi
done

# Backwards compatibility: window-manager -> babydra-window-manager.
for sdir in "${SYSTEM_BIN_DIRS[@]}"; do
    if [ -d "$sdir" ] && [ -x "$sdir/babydra-window-manager" ]; then
        if [ -w "$sdir" ]; then
            ln -sf babydra-window-manager "$sdir/window-manager" 2>/dev/null || true
        else
            run_privileged ln -sf babydra-window-manager "$sdir/window-manager" 2>/dev/null || true
        fi
    fi
done
if [ -x "$USER_BIN_DIR/babydra-window-manager" ]; then
    ln -sf babydra-window-manager "$USER_BIN_DIR/window-manager" 2>/dev/null || true
fi

# greetd service, config and TTY takeover are handled by 07-services.sh.
