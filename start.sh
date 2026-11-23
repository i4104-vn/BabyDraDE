#!/usr/bin/env bash
# BabyDraDE session entry point.
#
# Option parsing stays here; the actual work is split into numbered scripts in
# scripts/ and executed in order. Each step script sources scripts/common.sh.
set -eu

ROOT_DIR="$(cd "$(dirname "$0")" && pwd)"
export BABYDRADE_ROOT="$ROOT_DIR"
cd "$ROOT_DIR"

FORCE_INSTALL_DEPS=0
SKIP_DEPS=0
BABYDRADE_ORIENTATION="landscape"
BABYDRADE_TRANSFORM="90"
BABYDRADE_TOUCH_TRANSFORM=""
APP_ARGS=()

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
        --landscape)
            BABYDRADE_ORIENTATION="landscape"
            BABYDRADE_TRANSFORM="90"
            shift
            ;;
        --portrait)
            BABYDRADE_ORIENTATION="portrait"
            BABYDRADE_TRANSFORM="0"
            shift
            ;;
        --rotate)
            if [ $# -gt 1 ]; then
                BABYDRADE_TRANSFORM="$2"
                shift 2
            else
                shift
            fi
            ;;
        --touch-rotate)
            if [ $# -gt 1 ]; then
                BABYDRADE_TOUCH_TRANSFORM="$2"
                shift 2
            else
                shift
            fi
            ;;
        *)
            APP_ARGS+=("$1")
            shift
            ;;
    esac
done

export FORCE_INSTALL_DEPS SKIP_DEPS
export BABYDRADE_ORIENTATION BABYDRADE_TRANSFORM BABYDRADE_TOUCH_TRANSFORM

# Bash cannot export arrays, so forward arbitrary compositor arguments as
# numbered environment variables (preserves spaces inside an argument).
export BABYDRADE_APP_ARGS_COUNT="${#APP_ARGS[@]}"
arg_index=0
for arg in "${APP_ARGS[@]+"${APP_ARGS[@]}"}"; do
    export "BABYDRADE_APP_ARG_$arg_index=$arg"
    arg_index=$((arg_index + 1))
done

SCRIPTS=(
    "01-environment.sh"
    "02-gtk-theme.sh"
    "03-orientation.sh"
    "04-dependencies.sh"
    "05-session.sh"
    "06-build-install.sh"
    "07-launch.sh"
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
