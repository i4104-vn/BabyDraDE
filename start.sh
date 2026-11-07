#!/usr/bin/env bash
# ==============================================================================
# BabyDraDE — Startup & Launch Script from Linux TTY
# Tự động nhận diện hệ điều hành, cài đặt dependencies, cấu hình TTY
# và khởi chạy giao diện đồ họa trực tiếp để test.
# ==============================================================================

set -euo pipefail

# Chuyển về thư mục chứa source BabyDraDE
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# ------------------------------------------------------------------------------
# 1. Helper phân quyền root/sudo
# ------------------------------------------------------------------------------
run_sudo() {
    if [ "$(id -u)" -eq 0 ]; then
        "$@"
    elif command -v sudo >/dev/null 2>&1; then
        sudo "$@"
    elif command -v doas >/dev/null 2>&1; then
        doas "$@"
    else
        echo "[!] Không tìm thấy sudo/doas và bạn không phải root. Vui lòng chạy lại với quyền root."
        su -c "$*"
    fi
}

echo "=========================================================="
echo "          BabyDraDE — Wayland Desktop Environment         "
echo "=========================================================="

# ------------------------------------------------------------------------------
# 2. Tự động nhận diện hệ điều hành và cài đặt package phù hợp
# ------------------------------------------------------------------------------
detect_and_install_packages() {
    echo "[*] Đang nhận diện hệ điều hành..."

    OS_ID="unknown"
    OS_LIKE="unknown"

    if [ -f /etc/os-release ]; then
        . /etc/os-release
        OS_ID="${ID:-unknown}"
        OS_LIKE="${ID_LIKE:-unknown}"
    elif [ -f /etc/issue ]; then
        OS_ID="$(head -n1 /etc/issue | awk '{print tolower($1)}')"
    fi

    echo "[*] Phát hiện OS: $OS_ID (Family: $OS_LIKE)"

    case "$OS_ID" in
        ubuntu|debian|linuxmint|pop|kali|raspbian|devuan)
            echo "[*] Cài đặt dependencies cho Debian/Ubuntu..."
            export DEBIAN_FRONTEND=noninteractive
            run_sudo apt-get update -y
            run_sudo apt-get install -y \
                build-essential \
                pkg-config \
                cmake \
                curl \
                git \
                libwayland-dev \
                wayland-protocols \
                libxkbcommon-dev \
                libdrm-dev \
                libinput-dev \
                libseat-dev \
                libgbm-dev \
                libegl1-mesa-dev \
                libgles2-mesa-dev \
                mesa-utils \
                dbus \
                seatd \
                kbd \
                fbset \
                xorg \
                xinit \
                alacritty || true
            ;;

        arch|manjaro|endeavouros|artix|garuda)
            echo "[*] Cài đặt dependencies cho Arch Linux..."
            run_sudo pacman -Sy --noconfirm --needed \
                base-devel \
                pkgconf \
                cmake \
                curl \
                git \
                wayland \
                wayland-protocols \
                libxkbcommon \
                libdrm \
                libinput \
                seatd \
                mesa \
                dbus \
                fbset \
                xorg-server \
                xorg-xinit \
                alacritty || true
            ;;

        fedora|rhel|centos|rocky|alma)
            echo "[*] Cài đặt dependencies cho Fedora/RHEL..."
            PACKAGE_MGR="dnf"
            command -v dnf >/dev/null 2>&1 || PACKAGE_MGR="yum"
            run_sudo "$PACKAGE_MGR" install -y \
                gcc \
                gcc-c++ \
                make \
                cmake \
                pkgconf-pkg-config \
                curl \
                git \
                wayland-devel \
                wayland-protocols-devel \
                libxkbcommon-devel \
                libdrm-devel \
                libinput-devel \
                seatd-devel \
                mesa-libEGL-devel \
                mesa-libGLES-devel \
                dbus \
                fbset \
                xorg-x11-server-Xorg \
                xorg-x11-xinit \
                alacritty || true
            ;;

        alpine)
            echo "[*] Cài đặt dependencies cho Alpine Linux..."
            run_sudo apk update
            run_sudo apk add \
                build-base \
                cmake \
                pkgconf \
                curl \
                git \
                wayland-dev \
                wayland-protocols \
                libxkbcommon-dev \
                libdrm-dev \
                libinput-dev \
                seatd-dev \
                mesa-dev \
                dbus \
                rust \
                cargo || true
            ;;

        opensuse*|suse|sles)
            echo "[*] Cài đặt dependencies cho openSUSE..."
            run_sudo zypper --non-interactive install -y \
                devel_basis \
                cmake \
                pkg-config \
                curl \
                git \
                wayland-devel \
                wayland-protocols-devel \
                libxkbcommon-devel \
                libdrm-devel \
                libinput-devel \
                libseat-devel \
                Mesa-libEGL-devel \
                Mesa-libGLESv2-devel \
                dbus-1 || true
            ;;

        void)
            echo "[*] Cài đặt dependencies cho Void Linux..."
            run_sudo xbps-install -Sy \
                base-devel \
                pkg-config \
                cmake \
                curl \
                git \
                wayland-devel \
                libxkbcommon-devel \
                libdrm-devel \
                libinput-devel \
                seatd-devel \
                MesaLib-devel \
                dbus || true
            ;;

        *)
            echo "[!] Hệ điều hành chưa được cấu hình sẵn package manager riêng."
            echo "[*] Đang thử sử dụng apt-get / pacman / dnf nếu có..."
            if command -v apt-get >/dev/null 2>&1; then
                export DEBIAN_FRONTEND=noninteractive
                run_sudo apt-get update -y && run_sudo apt-get install -y build-essential pkg-config cmake curl git libdrm-dev libinput-dev libxkbcommon-dev || true
            elif command -v pacman >/dev/null 2>&1; then
                run_sudo pacman -Sy --noconfirm --needed base-devel pkgconf cmake curl git libdrm libinput libxkbcommon || true
            elif command -v dnf >/dev/null 2>&1; then
                run_sudo dnf install -y gcc gcc-c++ make cmake pkgconf-pkg-config curl git libdrm-devel libinput-devel libxkbcommon-devel || true
            fi
            ;;
    esac
}

# ------------------------------------------------------------------------------
# 3. Kiểm tra và cài đặt Rust / Cargo
# ------------------------------------------------------------------------------
ensure_rust_installed() {
    if ! command -v cargo >/dev/null 2>&1; then
        if [ -f "$HOME/.cargo/env" ]; then
            # Nạp environment cargo nếu đã cài nhưng chưa load PATH
            . "$HOME/.cargo/env"
        fi
    fi

    if ! command -v cargo >/dev/null 2>&1; then
        echo "[*] Không tìm thấy Cargo. Đang tự động cài đặt Rust qua rustup..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
        if [ -f "$HOME/.cargo/env" ]; then
            . "$HOME/.cargo/env"
        fi
    fi

    export PATH="$HOME/.cargo/bin:$PATH"
    echo "[✓] Rust toolchain: $(cargo --version 2>/dev/null || echo 'Cargo sẵn sàng')"
}

# ------------------------------------------------------------------------------
# 4. Chuẩn bị môi trường Wayland Runtime
# ------------------------------------------------------------------------------
setup_tty_environment() {
    echo "[*] Đang cấu hình môi trường Wayland Compositor..."

    # XDG_RUNTIME_DIR bắt buộc cho các session Wayland / DRM
    if [ -z "${XDG_RUNTIME_DIR:-}" ] || [ ! -d "${XDG_RUNTIME_DIR:-}" ]; then
        CURRENT_UID="$(id -u)"
        export XDG_RUNTIME_DIR="/tmp/runtime-$CURRENT_UID"
        mkdir -p "$XDG_RUNTIME_DIR"
        chmod 0700 "$XDG_RUNTIME_DIR"
    fi

    # Cấp quyền truy cập video / input / render cho user
    if [ "$(id -u)" -ne 0 ]; then
        for grp in video input render; do
            if getent group "$grp" >/dev/null 2>&1; then
                run_sudo usermod -a -G "$grp" "$USER" 2>/dev/null || true
            fi
        done
        run_sudo chmod 666 /dev/dri/card* /dev/dri/renderD* /dev/input/event* 2>/dev/null || true
    fi

    # D-Bus session bus
    if [ -z "${DBUS_SESSION_BUS_ADDRESS:-}" ]; then
        if command -v dbus-launch >/dev/null 2>&1; then
            eval "$(dbus-launch --sh-syntax)"
        fi
    fi

    # Thiết lập biến Wayland Display theo mục 3 trong planning.md
    export WAYLAND_DISPLAY="me.babydra.compositor"

    # Thiết lập config mặc định (~/.config/babydrade/config.toml) theo mục 3 trong planning.md
    USER_CONFIG_DIR="$HOME/.config/babydrade"
    if [ ! -f "$USER_CONFIG_DIR/config.toml" ]; then
        echo "[*] Khởi tạo config mặc định tại $USER_CONFIG_DIR/config.toml..."
        mkdir -p "$USER_CONFIG_DIR"
        if [ -f "$SCRIPT_DIR/config/default.toml" ]; then
            cp "$SCRIPT_DIR/config/default.toml" "$USER_CONFIG_DIR/config.toml"
        fi
    fi
}

# ------------------------------------------------------------------------------
# 5. Build Desktop Environment
# ------------------------------------------------------------------------------
build_babydrade() {
    echo "[*] Đang biên dịch BabyDraDE (release mode)..."
    cargo build --release --bin babydrade
}

# ------------------------------------------------------------------------------
# 6. Khởi chạy Compositor & Window Manager
# ------------------------------------------------------------------------------
launch_de() {
    echo "[*] Khởi chạy BabyDraDE Window Manager..."
    ./target/release/babydrade
}

# ------------------------------------------------------------------------------
# Thực thi toàn bộ chu trình thẳng tắp, không cần params
# ------------------------------------------------------------------------------
detect_and_install_packages
ensure_rust_installed
setup_tty_environment
build_babydrade
launch_de
