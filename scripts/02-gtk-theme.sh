#!/usr/bin/env bash
# 2. Synchronise the bundled GTK theme and window decorations.
set -eu
. "$(dirname "$0")/common.sh"

select_gtk_theme_variant() {
    local config_home="$1"
    local requested_theme="${BABYDRADE_GTK_THEME:-}"
    local settings_file

    case "$requested_theme" in
        dark|Dark|DARK)
            echo "dark"
            return
            ;;
        light|Light|LIGHT)
            echo "light"
            return
            ;;
        "")
            ;;
        *)
            echo "[!] Unknown BABYDRADE_GTK_THEME='$requested_theme'; using the saved preference or dark." >&2
            ;;
    esac

    for settings_file in "$config_home/gtk-4.0/settings.ini" "$config_home/gtk-3.0/settings.ini"; do
        if [ -f "$settings_file" ] && grep -Eq '^[[:space:]]*gtk-application-prefer-dark-theme[[:space:]]*=[[:space:]]*(1|true)[[:space:]]*$' "$settings_file"; then
            echo "dark"
            return
        elif [ -f "$settings_file" ] && grep -Eq '^[[:space:]]*gtk-application-prefer-dark-theme[[:space:]]*=[[:space:]]*(0|false)[[:space:]]*$' "$settings_file"; then
            echo "light"
            return
        fi
    done

    echo "dark"
}

sync_gtk_config() {
    if [ "${BABYDRADE_SKIP_GTK_SYNC:-0}" = "1" ]; then
        echo "[*] Skipping GTK configuration sync (BABYDRADE_SKIP_GTK_SYNC=1)."
        return
    fi

    local config_home
    if [ -n "${XDG_CONFIG_HOME:-}" ]; then
        config_home="$XDG_CONFIG_HOME"
    elif [ -n "${HOME:-}" ]; then
        config_home="$HOME/.config"
    else
        echo "[!] GTK configuration was not synced because HOME and XDG_CONFIG_HOME are unset." >&2
        return
    fi

    local theme_variant
    theme_variant="$(select_gtk_theme_variant "$config_home")"
    local source_dir="assets/gtk/$theme_variant"
    if [ ! -f "$source_dir/gtk.css" ] || [ ! -f "$source_dir/settings.ini" ]; then
        echo "[!] GTK theme assets are missing from $source_dir; skipping GTK configuration sync." >&2
        return
    fi

    local gtk_dir
    step "Syncing BabyDraDE $theme_variant GTK theme and titlebar controls..."
    for gtk_dir in "$config_home/gtk-3.0" "$config_home/gtk-4.0"; do
        mkdir -p "$gtk_dir"
        install -m 0644 "$source_dir/gtk.css" "$gtk_dir/gtk.css"
        install -m 0644 "$source_dir/settings.ini" "$gtk_dir/settings.ini"
    done
}

sync_gtk_config
