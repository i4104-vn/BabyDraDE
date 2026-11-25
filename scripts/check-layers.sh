#!/usr/bin/env bash
# Enforce the modular-monolith + Clean Architecture layout of window-manager.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SRC="$ROOT/crates/window-manager/src"
LIMIT=500
FAILED=0

check_absent() {
    local pattern="$1" message="$2"
    if grep -R -n -E "$pattern" "$SRC" --include='*.rs' >/dev/null 2>&1; then
        echo "[!] $message" >&2
        grep -R -n -E "$pattern" "$SRC" --include='*.rs' >&2 || true
        FAILED=1
    fi
}

# Legacy capability/layer roots must be gone.
check_absent 'crate::(adapters|domain|application|infrastructure|composition|backend|bootstrap|core::state)(::|[^a-zA-Z_])' \
    'legacy module roots must not be referenced'
check_absent '#\[path[[:space:]]*=' '#[path] redirection is forbidden'

for dir in app core infra modules shared; do
    [ -d "$SRC/$dir" ] || { echo "[!] missing $dir/" >&2; FAILED=1; }
done
for dir in input render shell wayland window blur; do
    [ -d "$SRC/modules/$dir" ] || { echo "[!] missing modules/$dir/" >&2; FAILED=1; }
done

# Every module owns a `models/` folder holding its types and impls.
for dir in app core infra shared; do
    found=$(find "$SRC/$dir" -type d -name models | head -1)
    [ -n "$found" ] || { echo "[!] missing $dir/models/" >&2; FAILED=1; }
done
for dir in input render wayland window blur; do
    [ -d "$SRC/modules/$dir/models" ] || { echo "[!] missing modules/$dir/models/" >&2; FAILED=1; }
done
for dir in launcher switcher lock screenshot; do
    found=$(find "$SRC/modules/shell/$dir" -type d -name models | head -1)
    [ -n "$found" ] || { echo "[!] missing modules/shell/$dir/models/" >&2; FAILED=1; }
done

while IFS= read -r file; do
    rel="${file#"$SRC/"}"
    lines=$(wc -l < "$file")
    if [ "$lines" -gt "$LIMIT" ]; then
        echo "[!] $rel has $lines lines (limit $LIMIT)" >&2
        FAILED=1
    fi
    depth=$(awk -F/ '{print NF-1}' <<< "$rel")
    max=5
    case "$rel" in
        modules/shell/screenshot/editor/*) max=7 ;;
    esac
    if [ "$depth" -gt "$max" ]; then
        echo "[!] $rel is depth $depth (max 5)" >&2
        FAILED=1
    fi
done < <(find "$SRC" -name '*.rs' -type f)

# `foo.rs` must not sit beside a `foo/` directory at the same level.
while IFS= read -r file; do
    base="${file%.rs}"
    if [ -d "$base" ]; then
        echo "[!] both $base.rs and $base/ exist" >&2
        FAILED=1
    fi
done < <(find "$SRC" -name '*.rs' -type f)

if [ "$FAILED" -ne 0 ]; then
    exit 1
fi

echo 'Modular monolith structure is satisfied.'
