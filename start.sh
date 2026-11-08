#!/usr/bin/env bash
set -eu

cd "$(dirname "$0")"

# 1. Ensure XDG_RUNTIME_DIR is properly configured (required for Wayland on TTY/ARM)
if [ -z "${XDG_RUNTIME_DIR:-}" ]; then
    CURRENT_UID="$(id -u 2>/dev/null || echo 1000)"
    export XDG_RUNTIME_DIR="/tmp/babydrade-runtime-$CURRENT_UID"
    mkdir -p "$XDG_RUNTIME_DIR"
    chmod 0700 "$XDG_RUNTIME_DIR"
fi

# 2. Check if a graphical host session (Wayland) is already active
if [ -n "${WAYLAND_DISPLAY:-}" ]; then
    echo "[*] Active Wayland session detected (WAYLAND_DISPLAY='${WAYLAND_DISPLAY}')."
    echo "[*] Launching BabyDraDE nested compositor..."
    exec cargo run "$@"
fi

# 3. Running directly from a bare Linux TTY / console
echo "[*] No active Wayland display found. Running directly on TTY."

# Use 'cage' Wayland kiosk runner with Pixman fallback (100% Wayland, no X11/xinit)
if command -v cage >/dev/null 2>&1; then
    if ! command -v Xwayland >/dev/null 2>&1; then
        echo "[!] 'cage' on Arch/Debian requires the Xwayland binary to initialize."
        echo "[!] Please install it with:"
        echo "    - Arch Linux: sudo pacman -S xorg-xwayland"
        echo "    - Debian / Ubuntu / Raspberry Pi OS: sudo apt install xwayland"
        echo ""
    fi
    echo "[*] Launching via Wayland 'cage' runner with software rasterizer (Pixman/llvmpipe) for universal hardware/ARM support..."
    export WLR_NO_HARDWARE_CURSORS=1
    export WLR_RENDERER=pixman
    export LIBGL_ALWAYS_SOFTWARE=1
    export EGL_LOG_LEVEL=debug
    export LIBGL_DEBUG=verbose
    exec cage -- cargo run "$@"
fi

echo "[!] Error: No Wayland session detected on TTY."
echo "[!] To launch BabyDraDE directly on TTY via Wayland, please install 'cage':"
echo "    - Arch Linux: sudo pacman -S cage"
echo "    - Debian / Ubuntu / Raspberry Pi OS: sudo apt install cage"
echo "[!] Then rerun: ./start.sh"
exit 1
