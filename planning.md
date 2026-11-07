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

## 4. Phím tắt mặc định

| Phím tắt | Hành động |
| :--- | :--- |
| `Super + Enter` | Mở terminal mặc định |
| `Super + Q` | Đóng cửa sổ đang focus |
| `Super + F` | Toggle Fullscreen |
| `Super + M` | Toggle Maximize |
| `Super + H` | Minimize (ẩn cửa sổ) |
| `Super + Tab` | Chuyển focus giữa các cửa sổ (Alt-Tab style) |
| `Super + 1..9` | Chuyển Workspace |
| `Super + Shift + 1..9` | Di chuyển cửa sổ sang Workspace |
| `Super + Shift + Q` | Thoát compositor |
| `Super + D` | Mở App Launcher (Phase sau) |

---

## 5. Các Phase triển khai

---

### Phase 0 — Skeleton & Black Screen (Tuần 1)

**Mục tiêu:** Workspace Cargo chạy được. Binary mở một cửa sổ đen (Winit backend).

- [ ] Khởi tạo Cargo workspace với 5 crate rỗng
- [ ] `main.rs`: Khởi tạo Smithay Winit backend → event loop → clear màn hình đen
- [ ] Logging: `tracing` + `tracing-subscriber` hoạt động
- [ ] `cargo build` thành công, `cargo run` hiện cửa sổ đen

**Milestone:** Cửa sổ đen xuất hiện, không crash, Ctrl+C để thoát.

---

### Phase 1 — Domain Entities & Traits (Tuần 2)

**Mục tiêu:** Định nghĩa toàn bộ dữ liệu cốt lõi và contracts. Không phụ thuộc thư viện ngoài.

- [ ] Entities: `Window`, `Workspace`, `Output`, `Cursor`
- [ ] Value Objects: `Rect`, `Point`, `Size`, `Color`, `KeyCombo`
- [ ] Traits: `CompositorBackend`, `CanvasRenderer`, `InputSource`, `ConfigStore`, `AppRegistry`
- [ ] Placement rules: `CascadePlacement` (mở cửa sổ mới lệch xuống-phải 30px so với cửa sổ trước), `CenterPlacement` (giữa màn hình)
- [ ] Focus rules: MRU (Most Recently Used) — khi đóng cửa sổ, focus quay về cửa sổ gần nhất
- [ ] Unit tests cho placement & focus logic

**Milestone:** `cargo test -p domain` — tất cả pass. Logic thuần túy, không cần GPU.

---

### Phase 2 — Wayland Compositor Core (Tuần 3–4)

**Mục tiêu:** App ngoài có thể kết nối vào compositor. Bắt được keyboard & pointer.

- [ ] Khởi tạo `wl_display`, bind globals (`wl_compositor`, `wl_shm`, `wl_seat`, `wl_output`)
- [ ] Implement `XdgShellHandler`: bắt `new_toplevel` → tạo `Window` entity, bắt `toplevel_destroyed`
- [ ] Implement `SeatHandler`: keyboard (xkbcommon) + pointer (motion, button, scroll)
- [ ] Bind Wayland socket tên `me.babydra.compositor`
- [ ] Winit backend hoạt động trơn tru cho dev
- [ ] DRM backend skeleton (chưa cần chạy thật)

**Milestone:** Chạy compositor, mở terminal gõ `WAYLAND_DISPLAY=me.babydra.compositor alacritty`. Log hiện "New toplevel: Alacritty". Chưa render lên màn hình.

---

### Phase 3 — Canvas Render + Window Textures (Tuần 5–7)

**Mục tiêu:** Nhìn thấy desktop Canvas + app ngoài render lên canvas + titlebar đầy đủ.

**Canvas nền:**
- [ ] Clear màn hình với solid color (từ config theme)
- [ ] Load ảnh wallpaper → GPU texture → vẽ full màn hình

**Render app ngoài:**
- [ ] Lấy `wl_buffer` từ client → import thành GPU texture
- [ ] Vẽ texture tại tọa độ `(window.x, window.y + titlebar_height)` với kích thước `(window.width, window.height)`
- [ ] Texture cache để không re-import mỗi frame

**Titlebar (Server-Side Decorations):**
- [ ] Vẽ thanh titlebar phía trên cửa sổ: màu nền + tên app (text rendering)
- [ ] 3 nút điều khiển bên phải: Minimize `[_]`, Maximize `[□]`, Close `[×]`
- [ ] Click nút → dispatch command tương ứng
- [ ] Kéo titlebar → di chuyển cửa sổ

**Border & Focus:**
- [ ] Viền cửa sổ: màu sáng khi focused, màu tối khi unfocused
- [ ] Click vào cửa sổ → raise to top + focus

**Resize:**
- [ ] Kéo 4 cạnh hoặc 4 góc → resize cửa sổ
- [ ] Gửi `configure` event với kích thước mới về cho app

**Scene Graph:**
- [ ] Render theo thứ tự: Background → Windows (z-order thấp → cao) → Cursor

**Milestone:** Chạy compositor, thấy nền desktop. Mở Alacritty → cửa sổ floating xuất hiện giữa màn hình với titlebar đầy đủ. Kéo thả, resize, đóng — tất cả hoạt động.

---

### Phase 4 — Window Management & Keybindings (Tuần 8–9)

**Mục tiêu:** Quản lý cửa sổ hoàn chỉnh, phím tắt, workspace, app launcher cơ bản.

**Keybinding System:**
- [ ] Đọc bảng keybinding từ config
- [ ] `InputDispatcher`: raw key event → tra bảng → dispatch command
- [ ] Implement tất cả phím tắt trong bảng ở mục 4

**Window States:**
- [ ] Maximize: cửa sổ chiếm toàn màn hình (ẩn titlebar tuỳ chọn), bấm lại → phục hồi vị trí cũ
- [ ] Minimize: ẩn cửa sổ, vẫn giữ trong danh sách
- [ ] Fullscreen: app request fullscreen (video player) → cửa sổ chiếm toàn màn hình, ẩn mọi thứ
- [ ] Cascade placement: mở cửa sổ mới lệch xuống-phải 30px

**Workspace:**
- [ ] Tối thiểu 4 workspace
- [ ] `Super + 1..4`: chuyển workspace (ẩn/hiện nhóm cửa sổ)
- [ ] `Super + Shift + 1..4`: chuyển cửa sổ đang focus sang workspace khác

**App Launcher (cơ bản):**
- [ ] Quét `.desktop` files từ `/usr/share/applications` và `~/.local/share/applications`
- [ ] `Super + D` → hiện overlay tìm kiếm đơn giản → chọn app → launch
- [ ] Spawn child process với `WAYLAND_DISPLAY=me.babydra.compositor`

**XWayland (tuỳ chọn):**
- [ ] Khởi tạo XWayland subprocess
- [ ] Map cửa sổ X11 → floating window trên canvas

**Milestone:** Mở nhiều app cùng lúc, kéo thả tự do, chuyển workspace, phím tắt hoạt động. App launcher mở được Firefox, Terminal, File Manager.

---

### Phase 5 — Config, Theme & CLI (Tuần 10–11)

**Mục tiêu:** Hệ thống cấu hình, theme, và CLI điều khiển.

**Config System:**
- [ ] Đọc `~/.config/babydrade/config.toml` khi khởi động
- [ ] Các mục: keybindings, border_width, titlebar_height, gaps, default_terminal, autostart
- [ ] Hot-reload: thay đổi file → áp dụng ngay

**Theme:**
- [ ] File theme riêng: màu nền, viền, titlebar, text, nút close/max/min
- [ ] 2 theme built-in: Dark (mặc định), Light

**drade-ctl CLI:**
- [ ] Giao tiếp compositor qua `me.babydra.ctl.sock`
- [ ] Lệnh: `drade-ctl window list`, `drade-ctl window close`, `drade-ctl workspace switch 2`, `drade-ctl reload`
- [ ] Output JSON

**Session:**
- [ ] Autostart: chạy danh sách app khi compositor khởi động
- [ ] Quit menu: `Super + Shift + Q` → thoát compositor sạch sẽ

**Milestone:** User có thể tuỳ chỉnh hoàn toàn qua config file. CLI điều khiển từ terminal.

---

### Phase 6 — Bar, Taskbar & Polish (Tuần 12+)

**Mục tiêu:** Thiết kế và xây dựng Bar / Taskbar theo ý tưởng riêng. Polish toàn bộ DE.

> Nội dung Phase này sẽ được lên ý tưởng sau khi các Phase trước hoàn thành.

**Dự kiến:**
- [ ] Top Bar hoặc Bottom Bar (vị trí & nội dung tuỳ ý)
- [ ] Taskbar / Dock hiển thị app đang mở
- [ ] System tray (clock, battery, wifi, volume)
- [ ] Notification system (D-Bus `org.freedesktop.Notifications`)
- [ ] Animation (fade, slide — tuỳ chọn tắt được)
- [ ] Context menu (click phải trên desktop)
- [ ] Lock screen

---

## 6. Tổng kết

| Phase | Thời gian | Kết quả |
| :--- | :--- | :--- |
| **0 — Skeleton** | 1 tuần | Cửa sổ đen chạy được |
| **1 — Domain** | 1 tuần | Entity + Trait + Unit tests pass |
| **2 — Compositor** | 2 tuần | App ngoài kết nối Wayland thành công |
| **3 — Canvas & Render** | 3 tuần | **Desktop hiển thị + app render + titlebar + kéo thả** |
| **4 — Window Mgmt** | 2 tuần | Keybinds + Workspace + App Launcher |
| **5 — Config & CLI** | 2 tuần | Config file + Theme + drade-ctl |
| **6 — Bar & Polish** | Tuỳ ý | Bar, Taskbar, Notifications (ý tưởng sau) |

**Tổng ước lượng Phase 0→5: ~11 tuần.** Phase 3 là milestone quan trọng nhất — khi hoàn thành bạn sẽ có một DE floating hoạt động được.

---

## 7. Thư viện Rust cốt lõi

| Thư viện | Mục đích |
| :--- | :--- |
| `smithay` | Wayland compositor framework |
| `calloop` | Event loop |
| `wayland-server` | Wayland protocol |
| `xkbcommon` | Keyboard layout |
| `image` | Decode wallpaper |
| `fontdue` / `cosmic-text` | Text rendering |
| `serde` + `toml` | Config |
| `nix` | Linux syscall |
| `clap` | CLI parser |
| `serde_json` | CLI output |

---

## 8. Lưu ý

- **Luôn dev trên Winit backend** (chạy trong cửa sổ). Chỉ chuyển DRM khi đã ổn định.
- **Smithay API chưa stable (0.x)** — ghim version cụ thể.
- **XWayland phức tạp** — có thể skip nếu không cần app X11 cũ.
- **Tham khảo:** [cosmic-comp](https://github.com/pop-os/cosmic-comp), [niri](https://github.com/YaLTeR/niri), [anvil](https://github.com/Smithay/smithay/tree/master/anvil).

