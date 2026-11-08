# BabyDraDE — Planning

> **Trạng thái:** Code Rust đã được reset để bắt đầu lại bằng MVP compositor nested trên Winit. Các checkbox được đặt lại vì trạng thái hoàn thành cũ không còn phản ánh mã nguồn hiện tại.

> **Mục tiêu:** Desktop Environment đơn giản trên Wayland bằng Rust. Canvas toàn màn hình làm nền, các app ngoài được render dưới dạng texture trong cửa sổ floating có titlebar đầy đủ.

---

## 1. Kiến trúc

### 1.1. Mô hình 4 lớp

| Lớp | Vai trò | Nội dung |
| :--- | :--- | :--- |
| **Presentation** | Vẽ mọi thứ lên màn hình | Canvas renderer, window frames (titlebar + border), cursor |
| **Application** | Điều phối logic | Mở/đóng/di chuyển/resize cửa sổ, focus, keybinding dispatch, launch app |
| **Domain** | Mô hình dữ liệu & quy tắc | Entity Window, Workspace; Traits trừu tượng; Floating placement rules |
| **Infrastructure** | Giao tiếp hệ thống | Smithay compositor, DRM/KMS, libinput, filesystem, IPC socket |

**Quy tắc phụ thuộc:** Presentation → Application → Domain ← Infrastructure. Domain không bao giờ import Infrastructure.

### 1.2. Luồng runtime

```
┌─────────────────────────────────────────────────────────┐
│                   BabyDraDE Process                     │
│                                                         │
│  calloop Event Loop (single thread)                     │
│    ├── Wayland Display Server (nhận client connections)  │
│    ├── Input Handler (keyboard + pointer từ libinput)    │
│    ├── IPC Socket (me.babydra.ctl)                      │
│    └── Render Loop (Canvas + Window textures → GPU)     │
└──────────────────────┬──────────────────────────────────┘
                       │
          ┌────────────┼────────────┐
          ▼            ▼            ▼
      Firefox      Alacritty    Legacy X11 App
   (Wayland client) (Wayland)   (qua XWayland)
```

### 1.3. Mô hình cửa sổ Floating

Mọi cửa sổ app ngoài đều là **floating window** với titlebar đầy đủ do BabyDraDE tự vẽ:

```
┌─────────────────────────────────────────────┐
│  [Icon] Tên App                  [_] [□] [×] │  ← Titlebar (minimize, maximize, close)
├─────────────────────────────────────────────┤
│                                             │
│         Nội dung app (GPU texture)          │
│                                             │
└─────────────────────────────────────────────┘
  ↑ Border: đổi màu khi focused/unfocused
  ↑ Drop shadow (tuỳ chọn)
  ↑ Resize handles ở 4 cạnh + 4 góc
```

Khi mở app mới → cửa sổ đặt ở **giữa màn hình** hoặc **cascade** (lệch xuống-phải so với cửa sổ trước). User tự do kéo thả, resize.

---

## 2. Cấu trúc thư mục

```
BabyDraDE/
│
├── Cargo.toml                          # Workspace root
├── planning.md                         # File này
├── README.md
├── rustfmt.toml
├── .editorconfig
│
├── config/
│   └── default.toml                    # Config mặc định (keybinds, colors, gaps)
│
├── assets/
│   ├── icons/                          # close.svg, maximize.svg, minimize.svg
│   ├── fonts/                          # Font mặc định cho titlebar
│   ├── wallpapers/                     # Ảnh nền mặc định
│   └── cursors/                        # Con trỏ chuột
│
│   ══════════════════════════════════════════════════
│   DOMAIN — Entities, Rules, Traits (không dependency ngoài)
│   ══════════════════════════════════════════════════
│
├── crates/
│   ├── domain/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── entities/
│   │       │   ├── mod.rs
│   │       │   ├── window.rs           # Window { id, title, app_id, geometry, state, focused }
│   │       │   ├── workspace.rs        # Workspace { id, name, windows, active_window }
│   │       │   ├── output.rs           # Output { id, resolution, scale }
│   │       │   └── cursor.rs           # Cursor { position, image }
│   │       │
│   │       ├── value_objects/
│   │       │   ├── mod.rs
│   │       │   ├── geometry.rs         # Rect, Point, Size, Edges
│   │       │   ├── color.rs            # Color (RGBA)
│   │       │   └── key_combo.rs        # KeyCombo { modifiers, key }
│   │       │
│   │       ├── rules/
│   │       │   ├── mod.rs
│   │       │   ├── placement.rs        # Cascade / Center placement khi mở cửa sổ mới
│   │       │   ├── focus.rs            # Focus policy: MRU khi đóng cửa sổ
│   │       │   └── constraints.rs      # Min/max size, aspect ratio
│   │       │
│   │       └── traits/
│   │           ├── mod.rs
│   │           ├── compositor.rs       # trait CompositorBackend
│   │           ├── renderer.rs         # trait CanvasRenderer
│   │           ├── input_source.rs     # trait InputSource
│   │           ├── config_store.rs     # trait ConfigStore
│   │           └── app_registry.rs     # trait AppRegistry
│   │
│   │   ════════════════════════════════════════════
│   │   APPLICATION — Services, Commands, Queries
│   │   ════════════════════════════════════════════
│   │
│   ├── application/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── services/
│   │       │   ├── mod.rs
│   │       │   ├── window_manager.rs   # open/close/move/resize/focus/maximize/minimize
│   │       │   ├── workspace_manager.rs # switch, move_window_to
│   │       │   ├── input_dispatcher.rs # raw input → keybinding → command
│   │       │   └── app_launcher.rs     # scan .desktop, launch app
│   │       │
│   │       ├── commands/
│   │       │   ├── mod.rs
│   │       │   ├── window_cmds.rs      # CloseWindow, MoveWindow, ResizeWindow, ToggleMaximize, Minimize
│   │       │   ├── workspace_cmds.rs   # SwitchWorkspace, MoveToWorkspace
│   │       │   └── system_cmds.rs      # LaunchApp, ReloadConfig, Quit
│   │       │
│   │       └── events/
│   │           ├── mod.rs
│   │           └── de_events.rs        # WindowOpened, WindowClosed, WindowFocused, ...
│   │
│   │   ═════════════════════════════════════════════════════
│   │   INFRASTRUCTURE — Smithay, DRM, libinput, Filesystem
│   │   ═════════════════════════════════════════════════════
│   │
│   ├── infrastructure/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── compositor/
│   │       │   ├── mod.rs
│   │       │   ├── wayland_server.rs   # wl_display, bind protocols
│   │       │   ├── xdg_shell.rs        # xdg_toplevel: cửa sổ desktop
│   │       │   └── xwayland.rs         # Bridge cho app X11 cũ
│   │       │
│   │       ├── backends/
│   │       │   ├── mod.rs
│   │       │   ├── winit_backend.rs    # Dev mode: chạy trong cửa sổ
│   │       │   └── drm_backend.rs      # Production: chiếm TTY, nói chuyện GPU
│   │       │
│   │       ├── input/
│   │       │   ├── mod.rs
│   │       │   ├── keyboard.rs         # xkbcommon key handling
│   │       │   └── pointer.rs          # click, motion, scroll
│   │       │
│   │       ├── rendering/
│   │       │   ├── mod.rs
│   │       │   ├── gl_renderer.rs      # OpenGL ES renderer
│   │       │   └── texture_cache.rs    # Cache GPU textures
│   │       │
│   │       ├── config/
│   │       │   ├── mod.rs
│   │       │   └── toml_loader.rs      # Đọc config.toml
│   │       │
│   │       ├── system/
│   │       │   ├── mod.rs
│   │       │   └── desktop_files.rs    # Quét .desktop files
│   │       │
│   │       └── ipc/
│   │           ├── mod.rs
│   │           └── unix_socket.rs      # me.babydra.ctl socket
│   │
│   │   ══════════════════════════════════════════════════════
│   │   PRESENTATION — Canvas, Window Frames, Render Pipeline
│   │   ══════════════════════════════════════════════════════
│   │
│   ├── presentation/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── canvas/
│   │       │   ├── mod.rs
│   │       │   └── background.rs       # Wallpaper / solid color nền
│   │       │
│   │       ├── window_frame/
│   │       │   ├── mod.rs
│   │       │   ├── titlebar.rs         # Tên app + nút [_] [□] [×]
│   │       │   ├── borders.rs          # Viền focus/unfocus
│   │       │   ├── resize_handles.rs   # 4 cạnh + 4 góc kéo resize
│   │       │   └── shadow.rs           # Drop shadow
│   │       │
│   │       └── render_pipeline/
│   │           ├── mod.rs
│   │           ├── scene_graph.rs      # Layer ordering: BG → Windows → Overlay → Cursor
│   │           └── compositor_loop.rs  # Vòng lặp render chính
│   │
│   │   ════════════════════════════
│   │   drade-ctl — CLI điều khiển
│   │   ════════════════════════════
│   │
│   └── drade-ctl/
│       ├── Cargo.toml
│       └── src/
│           └── main.rs                 # drade-ctl window close, workspace switch 2
│
├── src/
│   └── main.rs                         # Entry point: bootstrap, chọn backend, chạy loop
│
└── docs/
    └── architecture.md
```

### Phụ thuộc giữa crate

```
                    ┌──────────┐
              ┌────►│  domain  │◄────┐
              │     └──────────┘     │
              │          ▲           │
              │          │           │
       ┌──────┴────┐ ┌───┴───────┐  │
       │application│ │infra-     │  │
       └──────▲────┘ │structure  │  │
              │      └───────────┘  │
              │                     │
       ┌──────┴──────┐              │
       │presentation │──────────────┘
       └─────────────┘

    main.rs: import tất cả, thực hiện dependency injection
    drade-ctl: chỉ import domain (command types)
```

---

## 3. Naming & Identity

| Mục | Giá trị |
| :--- | :--- |
| Binary chính | `babydrade` |
| CLI tool | `drade-ctl` |
| Wayland socket | `me.babydra.compositor` |
| IPC socket path | `/run/user/{UID}/me.babydra.ctl.sock` |
| D-Bus service (nếu cần) | `me.babydra.Desktop` |
| Config path | `~/.config/babydrade/config.toml` |

---

## 4. Lưu ý

- **Luôn dev trên Winit backend** (chạy trong cửa sổ). Chỉ chuyển DRM khi đã ổn định.
- **Smithay API chưa stable (0.x)** — ghim version cụ thể.
- **XWayland phức tạp** — có thể skip nếu không cần app X11 cũ.
- **Tham khảo:** [cosmic-comp](https://github.com/pop-os/cosmic-comp), [niri](https://github.com/YaLTeR/niri), [anvil](https://github.com/Smithay/smithay/tree/master/anvil).

