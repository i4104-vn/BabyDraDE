#!/usr/bin/env bash
set -eu

cd "$(dirname "$0")"

# 1. Ensure XDG_RUNTIME_DIR is properly configured
if [ -z "${XDG_RUNTIME_DIR:-}" ]; then
    CURRENT_UID="$(id -u 2>/dev/null || echo 1000)"
    export XDG_RUNTIME_DIR="/tmp/babydrade-runtime-$CURRENT_UID"
    mkdir -p "$XDG_RUNTIME_DIR"
    chmod 0700 "$XDG_RUNTIME_DIR"
fi

# 2. Set default diagnostic logging level
if [ -z "${RUST_LOG:-}" ]; then
    export RUST_LOG="info,infrastructure=debug,babydrade=debug"
fi

# 3. Check execution mode
if [ -n "${WAYLAND_DISPLAY:-}" ] || [ -n "${DISPLAY:-}" ]; then
    echo "[*] Active graphical session detected (WAYLAND_DISPLAY='${WAYLAND_DISPLAY:-}' / DISPLAY='${DISPLAY:-}')."
    echo "[*] Launching BabyDraDE in nested mode (Winit backend with GlowRenderer)..."
else
    echo "[*] Running on bare TTY / hardware console."
    echo "[*] Launching autonomous BabyDraDE Udev/DRM hardware compositor with GlowRenderer (OpenGL ES via GBM)..."
fi

# 4. Build workspace components (ensuring topbar binary is compiled alongside compositor)
echo "[*] Building workspace components (compositor and topbar)..."
cargo build --workspace

# 5. Direct execution — autonomous compositor, no cage, no xinit needed
exec cargo run "$@"
