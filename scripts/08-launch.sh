#!/usr/bin/env bash
# 7. Launch the installed compositor from the system PATH.
set -eu
. "$(dirname "$0")/common.sh"

# System-installed binaries only: never fall back to a copy in target/.
export PATH="$USER_BIN_DIR:/usr/local/bin:/usr/bin:$PATH"
COMPOSITOR_BIN="$(command -v babydra-window-manager 2>/dev/null || true)"

if [ -z "$COMPOSITOR_BIN" ] || [ ! -x "$COMPOSITOR_BIN" ]; then
    echo "[!] babydra-window-manager was not found on PATH after installation." >&2
    exit 1
fi

case "$COMPOSITOR_BIN" in
    "$BABYDRADE_ROOT"/target/*)
        echo "[!] Refusing to launch a build-tree binary: $COMPOSITOR_BIN" >&2
        exit 1
        ;;
esac

step "Selected installed compositor binary: $COMPOSITOR_BIN"

app_args=()
app_args_count="${BABYDRADE_APP_ARGS_COUNT:-0}"
for ((i = 0; i < app_args_count; i++)); do
    arg_var="BABYDRADE_APP_ARG_$i"
    app_args+=("${!arg_var}")
done

cleanup_session() {
    trap - EXIT INT TERM
    echo "[*] Cleaning up BabyDraDE features and session processes..."
    pkill -TERM -f "babydra-(desktop|topbar|panel|screenshot|keymap)" 2>/dev/null || true
    sleep 0.2
    pkill -9 -f "babydra-(desktop|topbar|panel|screenshot|keymap)" 2>/dev/null || true
}

trap cleanup_session EXIT INT TERM

step "Launching BabyDraDE compositor: $COMPOSITOR_BIN"
set +e
"$COMPOSITOR_BIN" "${app_args[@]+"${app_args[@]}"}"
COMPOSITOR_EXIT_CODE=$?
set -e

cleanup_session
exit "$COMPOSITOR_EXIT_CODE"
