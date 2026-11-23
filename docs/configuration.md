# Cấu hình BabyDraDE

BabyDraDE dùng một file cấu hình chính cho các thiết lập trung tâm và nhiều file nhỏ
cho từng subsystem. Tất cả nằm dưới `~/.babydra/`.

## Cây thư mục

```text
~/.babydra/
├── babydra.conf          # cấu hình chính
├── configs/
│   ├── blur.toml         # [blur]
│   ├── clipboard.toml    # [clipboard]
│   ├── cursor.toml       # [cursor]
│   ├── desktop.toml      # [desktop]
│   ├── display.toml      # [display]
│   ├── desktop.json      # tọa độ icon desktop (debounced, không phải TOML)
│   ├── input.toml        # [mouse], [touchpad], [gestures]
│   ├── keymap.toml       # [shortcuts]
│   └── lockscreen.toml   # [lockscreen]
├── cursor/               # theme con trỏ
├── logs/                 # log ứng dụng
└── wallpaper/            # hình nền người dùng
```

## `babydra.conf` (chính)

Chỉ chứa các nhóm cấu hình trung tâm:

```toml
"1st" = false

[autostart]
commands = ["babydra-desktop", "babydra-topbar"]

[power]
profile = "balanced"
auto_saver_enabled = true
saver_threshold = 20
charge_limit = 80

[wallpaper]
current = "/home/user/.babydra/wallpaper/wallpaper.jpeg"
mode = "static"

[notification]
dnd = false
```

## Input method (fcitx5)

Compositor export sẵn các biến môi trường `GTK_IM_MODULE`, `QT_IM_MODULE`,
`SDL_IM_MODULE`, `INPUT_METHOD=fcitx` và `XMODIFIERS=@im=fcitx` cho mọi tiến
trình con. Giá trị do người dùng đặt trước đó được giữ nguyên.

Để dùng được bộ gõ, cần có daemon fcitx5 chạy trong phiên Wayland. Thêm lệnh
khởi động vào `[autostart]` nếu hệ thống của bạn chưa tự chạy:

```toml
[autostart]
commands = ["babydra-desktop", "babydra-topbar", "fcitx5"]
```

Các khóa không được nhận diện sẽ được giữ lại khi migration, nhưng không nên thêm
subsystem mới vào đây.

## File subsystem

Ví dụ `configs/input.toml`:

```toml
[mouse]
natural_scroll = false
speed = 1.0
middle_emulation = false

[touchpad]
tap_to_click = true
tap_drag = true
tap_drag_lock = false
button_map = "lrm"
dwt = true
natural_scroll = false
speed = 1.0

[gestures]
three_finger_swipe_window = true
swipe_threshold = 100.0
```

Mỗi file subsystem vẫn giữ tên section như trước (`[blur]`, `[cursor]`, ...), nên
nội dung dễ đọc và dễ chuyển từ bản cũ.

## Migration tự động

Khi `ensure_babydra_dirs()` chạy (lúc khởi động compositor/desktop/topbar):

1. Các section subsystem còn nằm trong `babydra.conf` cũ được chuyển sang file tương ứng
   trong `configs/`.
2. Nếu file đích đã có section đó, giá trị trong file đích **được ưu tiên** và section cũ
   bị xóa khỏi `babydra.conf`.
3. `babydra.conf` được ghi lại chỉ với các nhóm chính (cộng với khóa lạ do người dùng giữ lại).
4. `~/.babydra/keymap.toml` cũ được copy sang `configs/keymap.toml` nếu file mới chưa tồn tại.
5. Các file subsystem bị thiếu được tạo với giá trị mặc định.

Không có hành vi ghi đè dữ liệu người dùng trong file đích đã tồn tại.

## API cho lập trình viên

Dùng các hàm trong `services::config`, không đọc file trực tiếp:

- `load_babydra_config()` / `save_babydra_config()` — chỉ làm việc với cấu hình chính.
- `load_<subsystem>_config()` / `save_<subsystem>_config()` — làm việc với từng file nhỏ.
- `invalidate_cache()` — xóa toàn bộ cache khi cần đọc lại từ đĩa.
- `get_*_config_path()` — lấy đường dẫn cụ thể.

Việc đọc/ghi đi qua `config::store`, có cache theo mtime và ghi atomic (temp file + rename).
