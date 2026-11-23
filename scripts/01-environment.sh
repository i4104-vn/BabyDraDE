#!/usr/bin/env bash
# 1. Console & system English locale setup.
set -eu
. "$(dirname "$0")/common.sh"

configure_system_english() {
    if command -v loadkeys >/dev/null 2>&1; then
        loadkeys us >/dev/null 2>&1 || true
    fi

    local need_locale_update=0
    if [ -f /etc/locale.conf ] && ! grep -q "LANG=en_US.UTF-8" /etc/locale.conf 2>/dev/null; then
        need_locale_update=1
    fi

    if [ "$need_locale_update" -eq 1 ]; then
        step "Configuring system locale to English (en_US.UTF-8)..."
        if [ -f /etc/locale.gen ] && grep -q "#en_US.UTF-8 UTF-8" /etc/locale.gen 2>/dev/null; then
            run_privileged sed -i 's/^#en_US.UTF-8 UTF-8/en_US.UTF-8 UTF-8/' /etc/locale.gen 2>/dev/null || true
            if command -v locale-gen >/dev/null 2>&1; then
                run_privileged locale-gen 2>/dev/null || true
            fi
        fi

        if [ -f /etc/locale.conf ]; then
            run_privileged sh -c 'echo "LANG=en_US.UTF-8" > /etc/locale.conf' 2>/dev/null || true
        fi

        if [ -f /etc/vconsole.conf ] && ! grep -q "KEYMAP=us" /etc/vconsole.conf 2>/dev/null; then
            run_privileged sh -c 'echo "KEYMAP=us" >> /etc/vconsole.conf' 2>/dev/null || true
        fi
    fi
}

configure_system_english
