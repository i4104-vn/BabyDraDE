#!/usr/bin/env bash
set -eu

cd "$(dirname "$0")"

# ------------------------------------------------------------------------------
# 0. Parse arguments & options
# ------------------------------------------------------------------------------
FORCE_INSTALL_DEPS=0
SKIP_DEPS=0
BABYDRADE_ORIENTATION="landscape"
BABYDRADE_TRANSFORM="90"
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
        *)
            APP_ARGS+=("$1")
            shift
            ;;
    esac
done

# Ensure cargo is available in PATH if installed via rustup
if [ -n "${HOME:-}" ] && [ -f "$HOME/.cargo/env" ]; then
    # shellcheck source=/dev/null
    . "$HOME/.cargo/env"
fi

# ------------------------------------------------------------------------------
# 1. Console & System English Locale Setup
# ------------------------------------------------------------------------------
export LANG="en_US.UTF-8"
export LC_ALL="en_US.UTF-8"
export LANGUAGE="en_US:en"

run_privileged() {
    if [ "$(id -u)" -eq 0 ]; then
        "$@"
    elif command -v sudo >/dev/null 2>&1; then
        sudo "$@"
    elif command -v doas >/dev/null 2>&1; then
        doas "$@"
    else
        echo "[!] Root privileges required to execute command, but neither 'sudo' nor 'doas' was found."
        echo "[!] Please run the following command as root:"
        echo "    $*"
        exit 1
    fi
}

configure_system_english() {
    # Set virtual console keymap to US if in Linux TTY console
    if command -v loadkeys >/dev/null 2>&1; then
        loadkeys us >/dev/null 2>&1 || true
    fi

    # Check and configure system-wide English locale files if privileges permit
    local need_locale_update=0
    if [ -f /etc/locale.conf ] && ! grep -q "LANG=en_US.UTF-8" /etc/locale.conf 2>/dev/null; then
        need_locale_update=1
    fi

    if [ "$need_locale_update" -eq 1 ]; then
        echo "[*] Configuring system locale to English (en_US.UTF-8)..."
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

# ------------------------------------------------------------------------------
# 2. Screen Orientation Setup (Portrait -> Landscape)
# ------------------------------------------------------------------------------
configure_screen_orientation() {
    echo "[*] Setting screen orientation to $BABYDRADE_ORIENTATION (transform: $BABYDRADE_TRANSFORM)..."

    # 1. Hardware Framebuffer Console (TTY) rotation:
    # On tablet devices with native portrait screens, rotating fbcon switches console to landscape.
    # Rotation values for fbcon: 0 = 0°, 1 = 90° clockwise, 2 = 180°, 3 = 270° counter-clockwise.
    local fb_rotate=1
    if [ "$BABYDRADE_ORIENTATION" = "portrait" ]; then
        fb_rotate=0
    elif [ "$BABYDRADE_TRANSFORM" = "270" ]; then
        fb_rotate=3
    elif [ "$BABYDRADE_TRANSFORM" = "180" ]; then
        fb_rotate=2
    fi

    for fb_node in /sys/class/graphics/fbcon/rotate_all /sys/class/graphics/fbcon/rotate; do
        if [ -w "$fb_node" ]; then
            echo "$fb_rotate" > "$fb_node" 2>/dev/null || true
            break
        elif [ -f "$fb_node" ]; then
            run_privileged sh -c "echo '$fb_rotate' > '$fb_node'" 2>/dev/null || true
            break
        fi
    done

    # 2. Export environment variables for BabyDraDE compositor
    export BABYDRADE_ORIENTATION="$BABYDRADE_ORIENTATION"
    export BABYDRADE_TRANSFORM="$BABYDRADE_TRANSFORM"

    # 3. If running inside an active graphical session with wlr-randr or xrandr
    if [ -n "${WAYLAND_DISPLAY:-}" ] && command -v wlr-randr >/dev/null 2>&1; then
        local portrait_output
        portrait_output="$(wlr-randr 2>/dev/null | grep -B1 -E 'Transform: 90|Transform: 270' | grep -o '^[A-Za-z0-9_-]*' | head -n1 || true)"
        if [ -n "$portrait_output" ]; then
            echo "[*] Detected Wayland output '$portrait_output', ensuring landscape orientation..."
            wlr-randr --output "$portrait_output" --transform normal 2>/dev/null || true
        fi
    elif [ -n "${DISPLAY:-}" ] && command -v xrandr >/dev/null 2>&1; then
        xrandr -o normal >/dev/null 2>&1 || true
    fi
}

configure_screen_orientation

# ------------------------------------------------------------------------------
# 3. OS & Distribution Detection and Dependency Check / Installation
# ------------------------------------------------------------------------------
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

    # 1. Cargo (Rust toolchain)
    if ! command -v cargo >/dev/null 2>&1; then
        MISSING_DEPS+=("cargo (Rust toolchain)")
    fi

    # 2. C/C++ compiler
    if ! command -v cc >/dev/null 2>&1 && ! command -v gcc >/dev/null 2>&1 && ! command -v clang >/dev/null 2>&1; then
        MISSING_DEPS+=("C/C++ compiler (gcc or clang)")
    fi

    # 3. pkg-config / pkgconf
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
    echo "[*] Installing required packages for $OS_NAME ($family)..."

    case "$family" in
        arch)
            local pkgs=(
                base-devel
                pkgconf
                clang
                rust
                wayland
                wayland-protocols
                libxkbcommon
                systemd-libs
                libinput
                seatd
                libdrm
                mesa
                gtk4
                gtk4-layer-shell
            )
            run_privileged pacman -S --needed --noconfirm "${pkgs[@]}"
            ;;
        debian)
            local pkgs=(
                build-essential
                pkg-config
                clang
                libclang-dev
                rustc
                cargo
                libwayland-dev
                wayland-protocols
                libxkbcommon-dev
                libudev-dev
                libinput-dev
                libseat-dev
                libdrm-dev
                libgbm-dev
                libegl1-mesa-dev
                libgles2-mesa-dev
                libgtk-4-dev
                libgtk4-layer-shell-dev
            )
            run_privileged apt-get update
            run_privileged apt-get install -y --no-install-recommends "${pkgs[@]}"
            ;;
        fedora)
            local pkgs=(
                gcc
                gcc-c++
                make
                pkgconf-pkg-config
                clang
                clang-devel
                rust
                cargo
                wayland-devel
                wayland-protocols-devel
                libxkbcommon-devel
                systemd-devel
                libinput-devel
                libseat-devel
                libdrm-devel
                mesa-libgbm-devel
                mesa-libEGL-devel
                mesa-libGLES-devel
                gtk4-devel
                gtk4-layer-shell-devel
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
                patterns-devel-base-devel_basis
                gcc
                gcc-c++
                pkg-config
                clang
                rust
                cargo
                wayland-devel
                wayland-protocols-devel
                libxkbcommon-devel
                libudev-devel
                libinput-devel
                libseat-devel
                libdrm-devel
                Mesa-libgbm-devel
                Mesa-libEGL-devel
                Mesa-libGLESv2-devel
                gtk4-devel
                gtk4-layer-shell-devel
            )
            run_privileged zypper install -y --no-recommends "${pkgs[@]}"
            ;;
        alpine)
            local pkgs=(
                build-base
                pkgconf
                clang
                clang-dev
                rust
                cargo
                wayland-dev
                wayland-protocols
                libxkbcommon-dev
                eudev-dev
                libinput-dev
                seatd-dev
                libdrm-dev
                mesa-dev
                gtk4.0-dev
                gtk4-layer-shell-dev
            )
            run_privileged apk add --no-cache "${pkgs[@]}"
            ;;
        void)
            local pkgs=(
                base-devel
                pkg-config
                rust
                cargo
                wayland-devel
                wayland-protocols
                libxkbcommon-devel
                eudev-libudev-devel
                libinput-devel
                libseat-devel
                libdrm-devel
                MesaLib-devel
                gtk4-devel
                gtk4-layer-shell-devel
            )
            run_privileged xbps-install -Sy "${pkgs[@]}"
            ;;
        *)
            echo "[!] Unrecognized or unsupported distribution: $OS_NAME"
            echo "[*] Please make sure the following packages are installed on your system:"
            echo "    - Rust toolchain: cargo, rustc"
            echo "    - C/C++ compiler, pkg-config, clang"
            echo "    - Wayland: libwayland, wayland-protocols"
            echo "    - Smithay backend: libxkbcommon, libudev, libinput, libseat, libdrm, libgbm, EGL, GLESv2"
            echo "    - UI libraries: gtk4, gtk4-layer-shell"
            ;;
    esac
}

if [ "$SKIP_DEPS" -eq 0 ]; then
    detect_os
    echo "[*] Detected operating system: $OS_NAME (family: $DISTRO_FAMILY)"
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
fi

# ------------------------------------------------------------------------------
# 4. Ensure XDG_RUNTIME_DIR is properly configured
# ------------------------------------------------------------------------------
if [ -z "${XDG_RUNTIME_DIR:-}" ]; then
    CURRENT_UID="$(id -u 2>/dev/null || echo 1000)"
    export XDG_RUNTIME_DIR="/tmp/babydrade-runtime-$CURRENT_UID"
    mkdir -p "$XDG_RUNTIME_DIR"
    chmod 0700 "$XDG_RUNTIME_DIR"
fi

# ------------------------------------------------------------------------------
# 5. Set default diagnostic logging level
# ------------------------------------------------------------------------------
if [ -z "${RUST_LOG:-}" ]; then
    export RUST_LOG="info,infrastructure=debug,babydrade=debug"
fi

# ------------------------------------------------------------------------------
# 6. Check execution mode
# ------------------------------------------------------------------------------
if [ -n "${WAYLAND_DISPLAY:-}" ] || [ -n "${DISPLAY:-}" ]; then
    echo "[*] Active graphical session detected (WAYLAND_DISPLAY='${WAYLAND_DISPLAY:-}' / DISPLAY='${DISPLAY:-}')."
    echo "[*] Launching BabyDraDE in nested mode (Winit backend with GlowRenderer)..."
else
    echo "[*] Running on bare TTY / hardware console."
    echo "[*] Launching autonomous BabyDraDE Udev/DRM hardware compositor with GlowRenderer (OpenGL ES via GBM)..."
fi

# ------------------------------------------------------------------------------
# 7. Build unified BabyDraDE Desktop Environment binary
# ------------------------------------------------------------------------------
echo "[*] Building unified BabyDraDE Desktop Environment..."
cargo build

# ------------------------------------------------------------------------------
# 8. Direct execution — autonomous compositor with built-in widgets
# ------------------------------------------------------------------------------
exec cargo run "${APP_ARGS[@]+"${APP_ARGS[@]}"}"
