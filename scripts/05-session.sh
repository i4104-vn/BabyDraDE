#!/usr/bin/env bash
# 5. Hardware session preparation (groups and seat daemon) for the installed WM.
set -eu
. "$(dirname "$0")/common.sh"

if [ -n "${WAYLAND_DISPLAY:-}" ] || [ -n "${DISPLAY:-}" ]; then
    step "Graphical session detected; skipping TTY permission setup."
    exit 0
fi

step "Preparing bare TTY hardware access for the compositor..."
if [ "$(id -u)" -ne 0 ] && command -v id >/dev/null 2>&1; then
    USER_GROUPS="$(id -Gn 2>/dev/null || true)"
    for grp in video render seat input; do
        if getent group "$grp" >/dev/null 2>&1 && ! echo " $USER_GROUPS " | grep -q " $grp "; then
            echo "[*] Adding '${USER:-$(id -un)}' to '$grp' (DRM master / input access)."
            run_privileged usermod -aG "$grp" "${USER:-$(id -un)}" 2>/dev/null || true
        fi
    done
fi

if command -v systemctl >/dev/null 2>&1; then
    if systemctl is-enabled --quiet seatd 2>/dev/null && ! systemctl is-active --quiet seatd 2>/dev/null; then
        run_privileged systemctl start seatd 2>/dev/null || true
    fi
fi
