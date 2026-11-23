#!/usr/bin/env bash
# 3. Screen orientation setup (portrait -> landscape).
set -eu
. "$(dirname "$0")/common.sh"

BABYDRADE_ORIENTATION="${BABYDRADE_ORIENTATION:-landscape}"
BABYDRADE_TRANSFORM="${BABYDRADE_TRANSFORM:-90}"

configure_screen_orientation() {
    step "Setting screen orientation to $BABYDRADE_ORIENTATION (transform: $BABYDRADE_TRANSFORM)..."

    local fb_rotate=0
    case "$BABYDRADE_TRANSFORM" in
        90|flipped-90)
            fb_rotate=3
            ;;
        180|flipped-180)
            fb_rotate=2
            ;;
        270|flipped-270)
            fb_rotate=1
            ;;
        0|normal|flipped)
            fb_rotate=0
            ;;
        *)
            if [ "$BABYDRADE_ORIENTATION" = "landscape" ]; then
                fb_rotate=3
            else
                fb_rotate=0
            fi
            ;;
    esac

    local fb_node
    for fb_node in /sys/class/graphics/fbcon/rotate_all /sys/class/graphics/fbcon/rotate /sys/class/graphics/fb0/rotate; do
        if [ -w "$fb_node" ]; then
            echo "$fb_rotate" > "$fb_node" 2>/dev/null || true
        elif [ -f "$fb_node" ]; then
            run_privileged sh -c "echo '$fb_rotate' > '$fb_node'" 2>/dev/null || true
        fi
    done

    if [ -n "${WAYLAND_DISPLAY:-}" ] && command -v wlr-randr >/dev/null 2>&1; then
        local portrait_output
        portrait_output="$(wlr-randr 2>/dev/null | grep -B1 -E 'Transform: 90|Transform: 270' | grep -o '^[A-Za-z0-9_-]*' | head -n1 || true)"
        if [ -n "$portrait_output" ]; then
            step "Detected Wayland output '$portrait_output', ensuring landscape orientation..."
            wlr-randr --output "$portrait_output" --transform normal 2>/dev/null || true
        fi
    elif [ -n "${DISPLAY:-}" ] && command -v xrandr >/dev/null 2>&1; then
        xrandr -o normal >/dev/null 2>&1 || true
    fi
}

configure_screen_orientation
