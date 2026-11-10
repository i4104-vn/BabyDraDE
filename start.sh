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

git stash
git pull

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

    for fb_node in /sys/class/graphics/fbcon/rotate_all /sys/class/graphics/fbcon/rotate /sys/class/graphics/fb0/rotate; do
        if [ -w "$fb_node" ]; then
            echo "$fb_rotate" > "$fb_node" 2>/dev/null || true
        elif [ -f "$fb_node" ]; then
            run_privileged sh -c "echo '$fb_rotate' > '$fb_node'" 2>/dev/null || true
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
if [ -z "${XDG_RUNTIME_DIR:-}" ] || [ ! -d "$XDG_RUNTIME_DIR" ] || [ ! -w "$XDG_RUNTIME_DIR" ]; then
    CURRENT_UID="$(id -u 2>/dev/null || echo 1000)"
    export XDG_RUNTIME_DIR="/tmp/babydrade-runtime-$CURRENT_UID"
    mkdir -p "$XDG_RUNTIME_DIR"
    chmod 0700 "$XDG_RUNTIME_DIR"
fi

# Clean up any lingering widget or compositor processes from previous sessions
echo "[*] Ensuring clean state (terminating any lingering previous session processes)..."
pkill -TERM -f "babydra-(window-manager|desktop|topbar|panel|screenshot|keymap|lock|widgets)" 2>/dev/null || true
pkill -TERM -x widgets 2>/dev/null || true
pkill -TERM -x window-manager 2>/dev/null || true
pkill -9 -f "babydra-(window-manager|desktop|topbar|panel|screenshot|keymap|lock|widgets)" 2>/dev/null || true
pkill -9 -x widgets 2>/dev/null || true
pkill -9 -x window-manager 2>/dev/null || true

# If no active compositor is running, clean up stale wayland socket & lock files to avoid socket conflicts
if ! pgrep -f "babydra-window-manager|window-manager" >/dev/null 2>&1; then
    rm -f "$XDG_RUNTIME_DIR"/wayland-*.lock 2>/dev/null || true
    if [ -z "${WAYLAND_DISPLAY:-}" ]; then
        rm -f "$XDG_RUNTIME_DIR"/wayland-[0-9]* 2>/dev/null || true
    fi
fi

# ------------------------------------------------------------------------------
# 5. Set default diagnostic logging level
# ------------------------------------------------------------------------------
if [ -z "${RUST_LOG:-}" ]; then
    export RUST_LOG="info,window_manager=debug,services=debug,babydrade=debug"
fi

# ------------------------------------------------------------------------------
# 6. Check execution mode
# ------------------------------------------------------------------------------
if [ -n "${WAYLAND_DISPLAY:-}" ] || [ -n "${DISPLAY:-}" ]; then
    echo "[*] Active graphical session detected (WAYLAND_DISPLAY='${WAYLAND_DISPLAY:-}' / DISPLAY='${DISPLAY:-}')."
    echo "[*] Launching BabyDraDE in nested mode (Winit backend with GlowRenderer)..."
else
    echo "[*] Running on bare TTY / hardware console."
    # Ensure current user has video/render/input/seat group access for DRM master
    if [ "$(id -u)" -ne 0 ] && command -v id >/dev/null 2>&1; then
        USER_GROUPS="$(id -Gn 2>/dev/null || true)"
        for grp in video render seat input; do
            if getent group "$grp" >/dev/null 2>&1 && ! echo " $USER_GROUPS " | grep -q " $grp "; then
                echo "[!] Current user '$USER' is not in '$grp' group (required for direct DRM master & input access)."
                echo "[*] Adding '$USER' to '$grp'..."
                run_privileged usermod -aG "$grp" "$USER" 2>/dev/null || true
            fi
        done
    fi

    # Ensure seatd or systemd-logind is active if present
    if command -v systemctl >/dev/null 2>&1; then
        if systemctl is-enabled --quiet seatd 2>/dev/null && ! systemctl is-active --quiet seatd 2>/dev/null; then
            run_privileged systemctl start seatd 2>/dev/null || true
        fi
    fi

    echo "[*] Launching autonomous BabyDraDE Udev/DRM hardware compositor with GlowRenderer (OpenGL ES via GBM)..."
fi

# ------------------------------------------------------------------------------
# 7. Build BabyDraDE binaries (compositor & widgets)
# ------------------------------------------------------------------------------
echo "[*] Building BabyDraDE binaries (compositor & widgets)..."
cargo build --workspace

# ------------------------------------------------------------------------------
# 8. Install BabyDraDE binaries into system directories
# ------------------------------------------------------------------------------
echo "[*] Installing BabyDraDE binaries into system..."
USER_BIN_DIR="${HOME:-}/.local/bin"
SYSTEM_BIN_DIRS=("/usr/local/bin" "/usr/bin")

# Prepend ~/.local/bin and /usr/local/bin to PATH for current session and children
export PATH="${USER_BIN_DIR}:/usr/local/bin:${PATH}"

find_babydra_bin() {
    local bin_name="$1"
    for candidate in \
        "/usr/local/bin/$bin_name" \
        "/usr/bin/$bin_name" \
        "${USER_BIN_DIR}/$bin_name" \
        "$bin_name" \
        "target/debug/$bin_name"
    do
        if [ -x "$candidate" ] || command -v "$candidate" >/dev/null 2>&1; then
            echo "$candidate"
            return 0
        fi
    done
    echo "$bin_name"
    return 1
}

BINARIES=(
    "babydra-window-manager"
    "babydra-desktop"
    "babydra-topbar"
    "babydra-panel"
    "babydra-screenshot"
    "babydra-keymap"
    "babydra-lock"
    "babydra-widgets"
)

if [ -n "${USER_BIN_DIR}" ]; then
    mkdir -p "${USER_BIN_DIR}"
fi

for bin in "${BINARIES[@]}"; do
    bin_path="target/debug/$bin"
    if [ -f "$bin_path" ]; then
        # 1. Install to ~/.local/bin
        if [ -n "${USER_BIN_DIR}" ]; then
            cp -f "$bin_path" "${USER_BIN_DIR}/$bin"
            chmod 755 "${USER_BIN_DIR}/$bin"
        fi

        # 2. Install to system directories (/usr/local/bin, /usr/bin)
        for sdir in "${SYSTEM_BIN_DIRS[@]}"; do
            if [ -d "$sdir" ]; then
                if [ -w "$sdir" ]; then
                    install -m 755 "$bin_path" "$sdir/$bin" 2>/dev/null || true
                else
                    run_privileged install -m 755 "$bin_path" "$sdir/$bin" 2>/dev/null || true
                fi
            fi
        done
    fi
done

# Backwards compatibility: create symlink for window-manager -> babydra-window-manager
if [ -n "${USER_BIN_DIR}" ] && [ -x "${USER_BIN_DIR}/babydra-window-manager" ]; then
    ln -sf "babydra-window-manager" "${USER_BIN_DIR}/window-manager" 2>/dev/null || true
fi
for sdir in "${SYSTEM_BIN_DIRS[@]}"; do
    if [ -d "$sdir" ] && [ -x "$sdir/babydra-window-manager" ]; then
        if [ -w "$sdir" ]; then
            ln -sf "babydra-window-manager" "$sdir/window-manager" 2>/dev/null || true
        else
            run_privileged ln -sf "babydra-window-manager" "$sdir/window-manager" 2>/dev/null || true
        fi
    fi
done

# ------------------------------------------------------------------------------
# 9. Launch from system — autonomous compositor with autostarted widgets
# ------------------------------------------------------------------------------
COMPOSITOR_BIN="$(find_babydra_bin "babydra-window-manager")"

echo "[*] Selected BabyDraDE compositor binary: $COMPOSITOR_BIN"

# Background launcher for desktop widgets (babydra-desktop & babydra-topbar)
PARENT_WAYLAND_DISPLAY="${WAYLAND_DISPLAY:-}"
WIDGET_LAUNCHER_PID=""

launch_widgets_on_wayland() {
    (
        set +e
        echo "[*] Background widget launcher waiting for Wayland compositor socket in $XDG_RUNTIME_DIR..."
        local socket_found=""

        # Poll for the new Wayland socket created by BabyDraDE (up to 5 seconds)
        for _ in $(seq 1 100); do
            for sock in "$XDG_RUNTIME_DIR"/wayland-[0-9]*; do
                if [ -S "$sock" ]; then
                    local sock_name
                    sock_name="$(basename "$sock")"
                    # In nested mode, ignore parent compositor socket
                    if [ -n "$PARENT_WAYLAND_DISPLAY" ] && [ "$sock_name" = "$PARENT_WAYLAND_DISPLAY" ]; then
                        continue
                    fi
                    socket_found="$sock_name"
                    break 2
                fi
            done
            sleep 0.05
        done

        if [ -z "$socket_found" ]; then
            echo "[!] Timeout: Wayland compositor socket was not detected within 5 seconds."
            exit 1
        fi

        echo "[*] Wayland socket detected: $socket_found. Setting up environment..."
        export WAYLAND_DISPLAY="$socket_found"
        export GDK_BACKEND="wayland"
        export ELECTRON_OZONE_PLATFORM_HINT="auto"
        export MOZ_ENABLE_WAYLAND="1"
        export QT_QPA_PLATFORM="wayland;xcb"
        export XDG_CURRENT_DESKTOP="BabyDra"
        export XDG_SESSION_DESKTOP="BabyDra"
        export XDG_SESSION_TYPE="wayland"

        # Allow compositor event loop and Wayland globals to settle
        sleep 0.2

        DESKTOP_BIN="$(find_babydra_bin "babydra-desktop")"
        TOPBAR_BIN="$(find_babydra_bin "babydra-topbar")"

        echo "[*] Autostarting BabyDraDE desktop: $DESKTOP_BIN"
        "$DESKTOP_BIN" &
        DESKTOP_PID=$!

        echo "[*] Autostarting BabyDraDE topbar: $TOPBAR_BIN"
        "$TOPBAR_BIN" &
        TOPBAR_PID=$!

        # Wait for widget processes
        wait "$DESKTOP_PID" "$TOPBAR_PID" 2>/dev/null || true
    ) &
    WIDGET_LAUNCHER_PID=$!
}

cleanup_session() {
    trap - EXIT INT TERM
    echo "[*] Cleaning up BabyDraDE widgets and session processes..."
    if [ -n "${WIDGET_LAUNCHER_PID:-}" ]; then
        kill "$WIDGET_LAUNCHER_PID" 2>/dev/null || true
    fi
    pkill -TERM -f "babydra-(desktop|topbar|panel|screenshot|keymap|lock|widgets)" 2>/dev/null || true
    sleep 0.2
    pkill -9 -f "babydra-(desktop|topbar|panel|screenshot|keymap|lock|widgets)" 2>/dev/null || true
}

trap cleanup_session EXIT INT TERM

# Start the background widget launcher
launch_widgets_on_wayland

# Run compositor in foreground (preserves TTY focus, KMS/DRM master, and signal delivery)
echo "[*] Launching BabyDraDE compositor: $COMPOSITOR_BIN"
set +e
"$COMPOSITOR_BIN" "${APP_ARGS[@]+"${APP_ARGS[@]}"}"
COMPOSITOR_EXIT_CODE=$?
set -e

cleanup_session
exit "$COMPOSITOR_EXIT_CODE"
