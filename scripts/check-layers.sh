#!/usr/bin/env bash
# Enforce window-manager layer boundaries and source-size limits.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SRC="$ROOT/crates/window-manager/src"
BASELINE="$(dirname "$0")/check-layers-baseline.txt"
[ -f "$BASELINE" ] || BASELINE=/dev/null
LIMIT=200
FAILED=0
GRANDFATHERED=0

check_forbidden() {
    local pattern="$1" paths="$2" message="$3"
    if grep -R -n -E "$pattern" $paths --include='*.rs' >/dev/null 2>&1; then
        echo "[!] $message" >&2
        grep -R -n -E "$pattern" $paths --include='*.rs' >&2 || true
        FAILED=1
    fi
}

check_forbidden 'crate::infrastructure' "$SRC/adapters" \
    'adapters must not import infrastructure'

check_forbidden 'crate::composition' \
    "$SRC/domain $SRC/application $SRC/adapters $SRC/infrastructure" \
    'inner layers must not import composition'

check_forbidden 'crate::adapters|crate::infrastructure' \
    "$SRC/domain $SRC/application" \
    'domain/application must not import outer layers'

check_forbidden 'gtk4|smithay|wayland' "$SRC/domain" \
    'domain must be framework-free'

# Size limit is a ratchet: files already over the limit are grandfathered in
# check-layers-baseline.txt. Any new violation fails; shrink the list as files
# are split so the 200-line standard is eventually universal.
while IFS= read -r file; do
    lines=$(wc -l < "$file")
    if [ "$lines" -gt "$LIMIT" ]; then
        rel="${file#"$ROOT/"}"
        if grep -F -x -q -- "$rel" "$BASELINE"; then
            GRANDFATHERED=$((GRANDFATHERED + 1))
            continue
        fi
        echo "[!] $file has $lines lines (limit $LIMIT)" >&2
        FAILED=1
    fi
done < <(find "$SRC" -name '*.rs' -type f)

if [ "$FAILED" -ne 0 ]; then
    exit 1
fi

if [ "$GRANDFATHERED" -ne 0 ]; then
    echo "Note: $GRANDFATHERED file(s) over $LIMIT lines are grandfathered in scripts/check-layers-baseline.txt."
fi

echo "Layer boundaries and file-size limits are satisfied."
