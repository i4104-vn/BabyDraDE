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
# 4. Chuẩn bị môi trường đồ họa cho TTY
# ------------------------------------------------------------------------------
setup_tty_environment() {
    echo "[*] Đang cấu hình môi trường đồ họa TTY..."

    # XDG_RUNTIME_DIR bắt buộc cho các session Wayland / DRM
    if [ -z "${XDG_RUNTIME_DIR:-}" ] || [ ! -d "${XDG_RUNTIME_DIR:-}" ]; then
        CURRENT_UID="$(id -u)"
        export XDG_RUNTIME_DIR="/tmp/runtime-$CURRENT_UID"
        mkdir -p "$XDG_RUNTIME_DIR"
        chmod 0700 "$XDG_RUNTIME_DIR"
    fi

    # Kích hoạt module kernel cho chuột và uinput nếu có
    run_sudo modprobe mousedev 2>/dev/null || true
    run_sudo modprobe uinput 2>/dev/null || true

    # Cấp quyền truy cập video / input / framebuffer cho user
    if [ "$(id -u)" -ne 0 ]; then
        for grp in video input render; do
            if getent group "$grp" >/dev/null 2>&1; then
                run_sudo usermod -a -G "$grp" "$USER" 2>/dev/null || true
            fi
        done
        # Cấp quyền đọc/ghi cho framebuffer, DRM và chuột /dev/input/mice
        run_sudo chmod 666 /dev/fb0 2>/dev/null || true
        run_sudo chmod 666 /dev/dri/card* /dev/dri/renderD* 2>/dev/null || true
        run_sudo chmod 666 /dev/input/mice /dev/input/mouse* /dev/input/event* 2>/dev/null || true
    fi

    # Tắt nhấp nháy con trỏ text của Linux fbcon để tránh xung đột khung hình
    echo 0 | run_sudo tee /sys/class/graphics/fbcon/cursor_blink >/dev/null 2>&1 || true
    echo 0 | run_sudo tee /sys/module/vt/parameters/cur_default >/dev/null 2>&1 || true
    run_sudo setterm -cursor off -blank 0 -powerdown 0 -powersave off 2>/dev/null || true
    printf "\033[?25l" 2>/dev/null || true

    # Cấu hình và hiển thị thông số framebuffer
    if [ -e /dev/fb0 ]; then
        if command -v fbset >/dev/null 2>&1; then
            run_sudo fbset -depth 32 2>/dev/null || true
        fi
        echo "[*] Thông số Framebuffer (/dev/fb0):"
        echo "    - Virtual Size : $(cat /sys/class/graphics/fb0/virtual_size 2>/dev/null || echo 'N/A')"
        echo "    - Stride       : $(cat /sys/class/graphics/fb0/stride 2>/dev/null || echo 'N/A') bytes"
        echo "    - BPP          : $(cat /sys/class/graphics/fb0/bits_per_pixel 2>/dev/null || echo 'N/A') bits"
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
# 6. Khởi chạy DE trực tiếp với giao diện đồ họa
# ------------------------------------------------------------------------------
launch_de() {
    echo "[*] Đang khởi động BabyDraDE..."
    echo "[*] Phím tắt trong DE:"
    echo "    - [Enter]           : Mở Terminal"
    echo "    - [d]               : Bật/Tắt App Launcher"
    echo "    - [Tab]             : Đổi focus giữa các cửa sổ"
    echo "    - [Phím Mũi Tên]   : Di chuyển cửa sổ đang focus"
    echo "    - [m]               : Maximize / Restore cửa sổ"
    echo "    - [q]               : Đóng cửa sổ đang chọn"
    echo "    - [1..4]            : Chuyển Workspace 1..4"
    echo "    - [Esc]             : Thoát DE về lại TTY"
    echo "----------------------------------------------------------"

    # Lưu lại trạng thái terminal hiện tại
    OLD_TTY_SETTINGS=""
    if [ -t 0 ]; then
        OLD_TTY_SETTINGS="$(stty -g 2>/dev/null || true)"
        # Đưa TTY sang raw mode không echo để truyền trực tiếp phím bấm vào DE
        stty -echo -icanon min 1 time 0 2>/dev/null || true
        # Ẩn con trỏ text của console
        tput civis 2>/dev/null || true
    fi

    # Đảm bảo dọn dẹp và khôi phục console khi thoát
    cleanup() {
        if [ -n "$OLD_TTY_SETTINGS" ]; then
            stty "$OLD_TTY_SETTINGS" 2>/dev/null || stty sane 2>/dev/null || true
        else
            stty sane 2>/dev/null || true
        fi
        printf "\033[?25h" 2>/dev/null || true
        run_sudo setterm -cursor on 2>/dev/null || true
        echo 1 | run_sudo tee /sys/class/graphics/fbcon/cursor_blink >/dev/null 2>&1 || true
        tput cnorm 2>/dev/null || true
        clear 2>/dev/null || true
        echo "[✓] Đã thoát BabyDraDE và khôi phục TTY."
    }
    trap cleanup EXIT INT TERM

    # Xoá màn hình TTY trước khi vẽ
    clear 2>/dev/null || true

    # Khởi chạy Desktop Canvas đồ họa
    if [ -e /dev/fb0 ] && [ -w /dev/fb0 ]; then
        echo "[*] Khởi chạy trực tiếp lên Hardware Framebuffer Canvas (/dev/fb0)..."
        ./target/release/babydrade
    elif [ -n "${DISPLAY:-}" ] || [ -n "${WAYLAND_DISPLAY:-}" ]; then
        echo "[*] Khởi chạy trong graphical session..."
        ./target/release/babydrade
    elif command -v xinit >/dev/null 2>&1; then
        echo "[*] Khởi tạo Xorg session từ TTY để mở Canvas đồ họa..."
        xinit ./target/release/babydrade -- :0 -nolisten tcp 2>/dev/null || ./target/release/babydrade
    else
        ./target/release/babydrade
    fi
}

# ------------------------------------------------------------------------------
# Thực thi toàn bộ chu trình thẳng tắp, không cần params
# ------------------------------------------------------------------------------
detect_and_install_packages
ensure_rust_installed
setup_tty_environment
build_babydrade
launch_de
