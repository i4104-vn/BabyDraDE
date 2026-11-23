#!/usr/bin/env bash
# 5. Session runtime dir, stale process cleanup and hardware session checks.
set -eu
. "$(dirname "$0")/common.sh"

step "Ensuring clean state (terminating any lingering previous session processes)..."
pkill -TERM -f "babydra-(window-manager|desktop|topbar|screenshot|keymap)" 2>/dev/null || true
pkill -TERM -x window-manager 2>/dev/null || true
pkill -9 -f "babydra-(window-manager|desktop|topbar|screenshot|keymap)" 2>/dev/null || true
pkill -9 -x window-manager 2>/dev/null || true

if ! pgrep -f "babydra-window-manager|window-manager" >/dev/null 2>&1; then
    rm -f "$XDG_RUNTIME_DIR"/wayland-*.lock 2>/dev/null || true
    if [ -z "${WAYLAND_DISPLAY:-}" ]; then
        rm -f "$XDG_RUNTIME_DIR"/wayland-[0-9]* 2>/dev/null || true
    fi
fi

if [ -n "${WAYLAND_DISPLAY:-}" ] || [ -n "${DISPLAY:-}" ]; then
    step "Active graphical session detected (WAYLAND_DISPLAY='${WAYLAND_DISPLAY:-}' / DISPLAY='${DISPLAY:-}')."
    step "Launching BabyDraDE in nested mode (Winit backend with GlowRenderer)..."
else
    step "Running on bare TTY / hardware console."
    if [ "$(id -u)" -ne 0 ] && command -v id >/dev/null 2>&1; then
        USER_GROUPS="$(id -Gn 2>/dev/null || true)"
        for grp in video render seat input; do
            if getent group "$grp" >/dev/null 2>&1 && ! echo " $USER_GROUPS " | grep -q " $grp "; then
                echo "[!] Current user '${USER:-$(id -un)}' is not in '$grp' group (required for direct DRM master & input access)."
                step "Adding '${USER:-$(id -un)}' to '$grp'..."
                run_privileged usermod -aG "$grp" "$USER" 2>/dev/null || true
            fi
        done
    fi

    if command -v systemctl >/dev/null 2>&1; then
        if systemctl is-enabled --quiet seatd 2>/dev/null && ! systemctl is-active --quiet seatd 2>/dev/null; then
            run_privileged systemctl start seatd 2>/dev/null || true
        fi
    fi

    step "Launching autonomous BabyDraDE Udev/DRM hardware compositor with GlowRenderer (OpenGL ES via GBM)..."
fi
