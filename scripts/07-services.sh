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

# Only write a config when none exists (or when a previous BabyDraDE-managed
# config is present). An existing working distro config is never overwritten.
config_src="$BABYDRADE_ROOT/packaging/greetd/config.toml"
if [ -f "$config_src" ]; then
    needs_config=0
    if [ ! -f /etc/greetd/config.toml ]; then
        needs_config=1
    elif grep -q "Managed by BabyDraDE" /etc/greetd/config.toml 2>/dev/null; then
        needs_config=1
    fi

    if [ "$needs_config" -eq 1 ]; then
        greeter_bin="$(command -v babydra-greeter 2>/dev/null || true)"
        if [ -z "$greeter_bin" ]; then
            greeter_bin="/usr/local/bin/babydra-greeter"
        fi
        if command -v cage >/dev/null 2>&1; then
            greeter_cmd="cage -s -- $greeter_bin"
        else
            echo "[!] cage is missing; the layer-shell greeter needs a Wayland session." >&2
            greeter_cmd="$greeter_bin"
        fi

        temp_config="$(mktemp)"
        {
            echo "# Managed by BabyDraDE install.sh"
            echo
            echo "[terminal]"
            echo "vt = 1"
            echo
            echo "[default_session]"
            echo "command = \"$greeter_cmd\""
            echo "user = \"greeter\""
        } > "$temp_config"
        run_privileged install -m 0644 "$temp_config" /etc/greetd/config.toml 2>/dev/null || true
        rm -f "$temp_config"
    fi
fi

# Use the distro-provided greetd unit (it already owns tty1 via Conflicts and
# has the correct PAM/keyring settings). Add only a small drop-in when the unit
# does not explicitly claim the VT.
dropin_dir="/etc/systemd/system/greetd.service.d"
if ! systemctl cat greetd.service 2>/dev/null | grep -q "Conflicts=getty@tty1.service"; then
    step "Adding tty1 override to greetd.service..."
    run_privileged mkdir -p "$dropin_dir" 2>/dev/null || true
    temp_dropin="$(mktemp)"
    cat > "$temp_dropin" <<'EOF'
[Unit]
After=getty@tty1.service
Conflicts=getty@tty1.service
EOF
    run_privileged install -m 0644 "$temp_dropin" "$dropin_dir/10-babydra-tty1.conf" 2>/dev/null || true
    rm -f "$temp_dropin"
fi

if run_privileged systemctl daemon-reload 2>/dev/null; then
    if run_privileged systemctl enable greetd.service 2>/dev/null; then
        echo "[✓] greetd.service enabled as display-manager.service."
    else
        echo "[!] Could not enable greetd.service; enable it manually." >&2
    fi

    run_privileged systemctl set-default graphical.target 2>/dev/null || true

    if run_privileged systemctl restart greetd.service 2>/dev/null; then
        echo "[✓] greetd restarted and owns tty1."
    else
        echo "[!] greetd could not be restarted now; it will run on next boot." >&2
    fi
else
    echo "[!] systemctl daemon-reload failed; service setup skipped." >&2
fi
