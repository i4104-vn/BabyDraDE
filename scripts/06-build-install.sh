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

# greetd owns the login screen; the compositor is started by the greeter's
# StartSession call immediately after successful authentication.
if command -v systemctl >/dev/null 2>&1; then
    unit_src="$BABYDRADE_ROOT/packaging/systemd/greetd.service"
    if [ -f "$unit_src" ] && [ -d /etc/systemd/system ]; then
        step "Installing greetd service (starts the BabyDraDE greeter)..."
        if [ -w /etc/systemd/system ]; then
            install -m 0644 "$unit_src" /etc/systemd/system/greetd.service
        else
            run_privileged install -m 0644 "$unit_src" /etc/systemd/system/greetd.service 2>/dev/null || true
        fi
        if run_privileged systemctl daemon-reload 2>/dev/null &&
            run_privileged systemctl enable greetd.service 2>/dev/null; then
            echo "[✓] greetd.service enabled (display-manager alias)."
        else
            echo "[!] Could not enable greetd.service; enable it manually." >&2
        fi
    fi
fi

if [ -d /etc/greetd ] && [ ! -f /etc/greetd/config.toml ]; then
    config_src="$BABYDRADE_ROOT/packaging/greetd/config.toml"
    if [ -f "$config_src" ]; then
        step "Installing default greetd configuration..."
        if [ -w /etc/greetd ]; then
            install -m 0644 "$config_src" /etc/greetd/config.toml
        else
            run_privileged install -m 0644 "$config_src" /etc/greetd/config.toml 2>/dev/null || true
        fi
    fi
fi
