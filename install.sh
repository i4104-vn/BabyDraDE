#!/usr/bin/env bash
# BabyDraDE installer.
#
# Installs packages, builds the workspace, installs the binaries into system
# directories and enables the required services. It does NOT launch the desktop
# session: greetd owns the login screen and starts the compositor after login.
set -eu

ROOT_DIR="$(cd "$(dirname "$0")" && pwd)"
export BABYDRADE_ROOT="$ROOT_DIR"
cd "$ROOT_DIR"

FORCE_INSTALL_DEPS=0
SKIP_DEPS=0

while [ $# -gt 0 ]; do
    case "$1" in
        --install-deps|-i)
            FORCE_INSTALL_DEPS=1
            shift
            ;;
        --skip-deps)
            SKIP_DEPS=1
            shift
            ;;
        *)
            echo "[!] Ignoring unknown option: $1" >&2
            shift
            ;;
    esac
done

export FORCE_INSTALL_DEPS SKIP_DEPS

SCRIPTS=(
    "01-environment.sh"
    "02-gtk-theme.sh"
    "04-dependencies.sh"
    "05-session.sh"
    "06-build-install.sh"
    "07-services.sh"
)

for script in "${SCRIPTS[@]}"; do
    echo "==> $script"
    set +e
    "$ROOT_DIR/scripts/$script"
    step_exit=$?
    set -e
    if [ "$step_exit" -ne 0 ]; then
        echo "[!] $script failed with exit code $step_exit" >&2
        exit "$step_exit"
    fi
done

echo "[✓] BabyDraDE installed. The session is started by greetd on tty1 after login."
