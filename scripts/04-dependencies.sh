#!/usr/bin/env bash
# 4. OS detection and dependency check / installation.
set -eu
. "$(dirname "$0")/common.sh"

FORCE_INSTALL_DEPS="${FORCE_INSTALL_DEPS:-0}"
SKIP_DEPS="${SKIP_DEPS:-0}"

detect_os() {
    OS_NAME="Unknown Linux"
    OS_ID=""
    OS_ID_LIKE=""

    local os_file=""
    if [ -n "${OS_RELEASE_FILE:-}" ] && [ -f "$OS_RELEASE_FILE" ]; then
        os_file="$OS_RELEASE_FILE"
    elif [ -f /etc/os-release ]; then
        os_file="/etc/os-release"
    elif [ -f /usr/lib/os-release ]; then
        os_file="/usr/lib/os-release"
    fi

    if [ -n "$os_file" ]; then
        OS_NAME="$(grep -E '^PRETTY_NAME=' "$os_file" | head -n1 | cut -d= -f2- | tr -d '"')"
        OS_ID="$(grep -E '^ID=' "$os_file" | head -n1 | cut -d= -f2- | tr -d '"')"
        OS_ID_LIKE="$(grep -E '^ID_LIKE=' "$os_file" | head -n1 | cut -d= -f2- | tr -d '"')"
    fi

    if [ -z "$OS_NAME" ]; then
        OS_NAME="${OS_ID:-Generic Linux}"
    fi

    DISTRO_FAMILY="unknown"
    case " ${OS_ID:-} ${OS_ID_LIKE:-} " in
        *" arch "*|*" manjaro "*|*" endeavouros "*|*" garuda "*|*" artix "*)
            DISTRO_FAMILY="arch"
            ;;
        *" debian "*|*" ubuntu "*|*" pop "*|*" linuxmint "*|*" kali "*|*" raspbian "*|*" zorin "*)
            DISTRO_FAMILY="debian"
            ;;
        *" fedora "*|*" rhel "*|*" centos "*|*" rocky "*|*" almalinux "*|*" nobara "*)
            DISTRO_FAMILY="fedora"
            ;;
        *" suse "*|*" opensuse "*|*" opensuse-tumbleweed "*|*" opensuse-leap "*)
            DISTRO_FAMILY="suse"
            ;;
        *" alpine "*)
            DISTRO_FAMILY="alpine"
            ;;
        *" void "*)
            DISTRO_FAMILY="void"
            ;;
        *)
            if command -v pacman >/dev/null 2>&1; then
                DISTRO_FAMILY="arch"
            elif command -v apt-get >/dev/null 2>&1; then
                DISTRO_FAMILY="debian"
            elif command -v dnf5 >/dev/null 2>&1 || command -v dnf >/dev/null 2>&1 || command -v yum >/dev/null 2>&1; then
                DISTRO_FAMILY="fedora"
            elif command -v zypper >/dev/null 2>&1; then
                DISTRO_FAMILY="suse"
            elif command -v apk >/dev/null 2>&1; then
                DISTRO_FAMILY="alpine"
            elif command -v xbps-install >/dev/null 2>&1; then
                DISTRO_FAMILY="void"
            fi
            ;;
    esac
}

check_missing_deps() {
    MISSING_DEPS=()

    if ! command -v cargo >/dev/null 2>&1; then
        MISSING_DEPS+=("cargo (Rust toolchain)")
    fi

    if ! command -v cc >/dev/null 2>&1 && ! command -v gcc >/dev/null 2>&1 && ! command -v clang >/dev/null 2>&1; then
        MISSING_DEPS+=("C/C++ compiler (gcc or clang)")
    fi

    if ! command -v wl-copy >/dev/null 2>&1; then
        MISSING_DEPS+=("wl-clipboard (wl-copy / wl-paste)")
    fi

    if ! command -v pkg-config >/dev/null 2>&1 && ! command -v pkgconf >/dev/null 2>&1; then
        MISSING_DEPS+=("pkg-config / pkgconf")
    else
        local modules=(
            "wayland-server"
            "wayland-client"
            "wayland-protocols"
            "xkbcommon"
            "libudev"
            "libinput"
            "libseat"
            "libdrm"
            "gbm"
            "egl"
            "glesv2"
            "gtk4"
            "gtk4-layer-shell-0"
        )
        for mod in "${modules[@]}"; do
            if ! pkg-config --exists "$mod" 2>/dev/null; then
                MISSING_DEPS+=("pkg-config module: $mod")
            fi
        done
    fi
}

install_packages_for_distro() {
    local family="$1"
    step "Installing required packages for $OS_NAME ($family)..."

    case "$family" in
        arch)
            local pkgs=(
                base-devel pkgconf clang rust wayland wayland-protocols
                libxkbcommon systemd-libs libinput seatd libdrm mesa gtk4
                gtk4-layer-shell pipewire pipewire-pulse wireplumber wl-clipboard
            )
            run_privileged pacman -S --needed --noconfirm "${pkgs[@]}"
            ;;
        debian)
            local pkgs=(
                build-essential pkg-config clang libclang-dev rustc cargo
                libwayland-dev wayland-protocols libxkbcommon-dev libudev-dev
                libinput-dev libseat-dev libdrm-dev libgbm-dev libegl1-mesa-dev
                libgles2-mesa-dev libgtk-4-dev libgtk4-layer-shell-dev
                pipewire pipewire-pulse wireplumber wl-clipboard
            )
            run_privileged apt-get update
            run_privileged apt-get install -y --no-install-recommends "${pkgs[@]}"
            ;;
        fedora)
            local pkgs=(
                gcc gcc-c++ make pkgconf-pkg-config clang clang-devel rust cargo
                wayland-devel wayland-protocols-devel libxkbcommon-devel
                systemd-devel libinput-devel libseat-devel libdrm-devel
                mesa-libgbm-devel mesa-libEGL-devel mesa-libGLES-devel
                gtk4-devel gtk4-layer-shell-devel pipewire pipewire-pulseaudio
                wireplumber wl-clipboard
            )
            if command -v dnf5 >/dev/null 2>&1; then
                run_privileged dnf5 install -y "${pkgs[@]}"
            elif command -v dnf >/dev/null 2>&1; then
                run_privileged dnf install -y "${pkgs[@]}"
            else
                run_privileged yum install -y "${pkgs[@]}"
            fi
            ;;
        suse)
            local pkgs=(
                patterns-devel-base-devel_basis gcc gcc-c++ pkg-config clang
                rust cargo wayland-devel wayland-protocols-devel libxkbcommon-devel
                libudev-devel libinput-devel libseat-devel libdrm-devel
                Mesa-libgbm-devel Mesa-libEGL-devel Mesa-libGLESv2-devel
                gtk4-devel gtk4-layer-shell-devel pipewire pipewire-pulse
                wireplumber wl-clipboard
            )
            run_privileged zypper install -y --no-recommends "${pkgs[@]}"
            ;;
        alpine)
            local pkgs=(
                build-base pkgconf clang clang-dev rust cargo wayland-dev
                wayland-protocols libxkbcommon-dev eudev-dev libinput-dev
                seatd-dev libdrm-dev mesa-dev gtk4.0-dev gtk4-layer-shell-dev
                pipewire pipewire-pulse wireplumber wl-clipboard
            )
            run_privileged apk add --no-cache "${pkgs[@]}"
            ;;
        void)
            local pkgs=(
                base-devel pkg-config rust cargo wayland-devel wayland-protocols
                libxkbcommon-devel eudev-libudev-devel libinput-devel libseat-devel
                libdrm-devel MesaLib-devel gtk4-devel gtk4-layer-shell-devel
                pipewire pipewire-pulse wireplumber wl-clipboard
            )
            run_privileged xbps-install -Sy "${pkgs[@]}"
            ;;
        *)
            echo "[!] Unrecognized or unsupported distribution: $OS_NAME" >&2
            echo "[*] Please make sure the following packages are installed on your system:" >&2
            echo "    - Rust toolchain: cargo, rustc" >&2
            echo "    - C/C++ compiler, pkg-config, clang" >&2
            echo "    - Wayland: libwayland, wayland-protocols" >&2
            echo "    - Smithay backend: libxkbcommon, libudev, libinput, libseat, libdrm, libgbm, EGL, GLESv2" >&2
            echo "    - UI libraries: gtk4, gtk4-layer-shell" >&2
            echo "    - Clipboard utilities: wl-clipboard (wl-copy / wl-paste)" >&2
            ;;
    esac
}

# greetd and cage are installed independently of the build toolchain so a
# missing package never aborts the rest of the setup.
install_login_manager_packages() {
    local family="$1"
    step "Installing greetd and cage (login screen session)..."
    case "$family" in
        arch)
            if run_privileged pacman -S --needed --noconfirm greetd cage; then
                echo "[✓] greetd and cage installed."
            else
                echo "[!] Could not install greetd/cage via pacman." >&2
            fi
            ;;
        debian)
            if run_privileged apt-get install -y --no-install-recommends greetd cage; then
                echo "[✓] greetd and cage installed."
            else
                echo "[!] greetd/cage are unavailable in this Debian repository." >&2
            fi
            ;;
        fedora)
            if run_privileged dnf5 install -y greetd cage 2>/dev/null ||
                run_privileged dnf install -y greetd cage 2>/dev/null; then
                echo "[✓] greetd and cage installed."
            else
                echo "[!] Could not install greetd/cage via dnf." >&2
            fi
            ;;
        alpine)
            if run_privileged apk add --no-cache greetd cage; then
                echo "[✓] greetd and cage installed."
            else
                echo "[!] Could not install greetd/cage via apk." >&2
            fi
            ;;
        void)
            if run_privileged xbps-install -Sy greetd cage; then
                echo "[✓] greetd and cage installed."
            else
                echo "[!] Could not install greetd/cage via xbps." >&2
            fi
            ;;
        *)
            echo "[!] No greetd package rule for $OS_NAME; install greetd and cage manually." >&2
            ;;
    esac
}

if [ "$SKIP_DEPS" -eq 0 ]; then
    detect_os
    step "Detected operating system: $OS_NAME (family: $DISTRO_FAMILY)"
    check_missing_deps

    if [ "${#MISSING_DEPS[@]}" -gt 0 ] || [ "$FORCE_INSTALL_DEPS" -eq 1 ]; then
        if [ "${#MISSING_DEPS[@]}" -gt 0 ]; then
            echo "[!] Missing ${#MISSING_DEPS[@]} required dependencies to build:"
            for dep in "${MISSING_DEPS[@]}"; do
                echo "    - $dep"
            done
        else
            echo "[*] Dependency installation option (--install-deps) is enabled."
        fi

        install_packages_for_distro "$DISTRO_FAMILY"

        if [ -n "${HOME:-}" ] && [ -f "$HOME/.cargo/env" ]; then
            # shellcheck source=/dev/null
            . "$HOME/.cargo/env"
        fi

        check_missing_deps
        if [ "${#MISSING_DEPS[@]}" -gt 0 ]; then
            echo "[!] Warning: Some dependencies are still unsatisfied:"
            for dep in "${MISSING_DEPS[@]}"; do
                echo "    - $dep"
            done
            echo "[*] If build fails, please verify and install the packages manually."
        else
            echo "[✓] All dependencies have been installed and verified."
        fi
    else
        echo "[✓] All required build dependencies are already satisfied."
    fi

    install_login_manager_packages "$DISTRO_FAMILY"
fi
