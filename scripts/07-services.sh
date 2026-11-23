#!/usr/bin/env bash
# 7. Install and enable greetd as the display manager owning the login TTY.
set -eu
. "$(dirname "$0")/common.sh"

if ! command -v systemctl >/dev/null 2>&1; then
    echo "[*] systemctl unavailable; skipping service setup."
    exit 0
fi

if ! command -v greetd >/dev/null 2>&1; then
    echo "[!] greetd is not installed; skipping service setup." >&2
    exit 0
fi

step "Configuring greetd display manager on tty1..."

# greetd runs the greeter as an unprivileged dedicated user.
if ! id greeter >/dev/null 2>&1; then
    run_privileged useradd --system --home-dir /var/lib/greetd \
        --shell /usr/sbin/nologin greeter 2>/dev/null || true
fi
run_privileged mkdir -p /etc/greetd /var/lib/greetd 2>/dev/null || true

config_src="$BABYDRADE_ROOT/packaging/greetd/config.toml"
if [ -f "$config_src" ]; then
    if [ -f /etc/greetd/config.toml ]; then
        run_privileged cp -f /etc/greetd/config.toml /etc/greetd/config.toml.bak 2>/dev/null || true
    fi
    run_privileged install -m 0644 "$config_src" /etc/greetd/config.toml 2>/dev/null || true
fi

unit_src="$BABYDRADE_ROOT/packaging/systemd/greetd.service"
if [ -f "$unit_src" ] && [ -d /etc/systemd/system ]; then
    run_privileged install -m 0644 "$unit_src" /etc/systemd/system/greetd.service 2>/dev/null || true

    # Point the unit at the actual greetd binary when the distro installs it
    # somewhere other than /usr/bin.
    greetd_bin="$(command -v greetd)"
    if [ "$greetd_bin" != "/usr/bin/greetd" ]; then
        run_privileged sed -i "s|ExecStart=/usr/bin/greetd|ExecStart=$greetd_bin|" \
            /etc/systemd/system/greetd.service 2>/dev/null || true
    fi
fi

if run_privileged systemctl daemon-reload 2>/dev/null; then
    if run_privileged systemctl enable greetd.service 2>/dev/null; then
        echo "[✓] greetd.service enabled as display-manager.service."
    else
        echo "[!] Could not enable greetd.service; enable it manually." >&2
    fi

    # Boot into the graphical target so greetd owns the VT at startup.
    run_privileged systemctl set-default graphical.target 2>/dev/null || true

    # Free tty1 from the regular getty: the unit also declares Conflicts so
    # starting greetd stops any getty already attached to that VT.
    run_privileged systemctl disable --now getty@tty1.service 2>/dev/null || true

    if run_privileged systemctl enable --now greetd.service 2>/dev/null; then
        echo "[✓] greetd is active and owns tty1."
    else
        echo "[!] greetd could not be started now; it will run on next boot." >&2
    fi
else
    echo "[!] systemctl daemon-reload failed; service setup skipped." >&2
fi
