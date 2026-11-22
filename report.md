# Báo cáo đánh giá kiến trúc `crates/window-manager`

> **Trạng thái:** báo cáo chính thức, đặt tại thư mục gốc dự án (`report.md`).
>
> **Phạm vi:** toàn bộ 99 file Rust trong `crates/window-manager/src`, các manifest liên quan,
> `docs/architecture.md`, `README.md`, và cách crate tương tác với các libs trong workspace.
>
> **Ngày đánh giá:** 2026-11-22.

---

## 1. Tóm tắt điều hành

`crates/window-manager` là một compositor Wayland viết trên Smithay 0.7, đồng thời đảm nhận
window manager, renderer, input router, shell UI (launcher, switcher, lock screen, screenshot
editor, clipboard picker), và một phần tích hợp hệ thống. Crate chạy được, `cargo check` sạch,
và đã có một số unit test. Tuy nhiên, kiến trúc hiện tại **không có biên giới thực sự enforceable**:

- Có **chu trình phụ thuộc hai chiều** giữa `compositor` ↔ `rendering` và
  `compositor` ↔ `extensions`.
- `BabydraState` là **god object** chứa gần như toàn bộ state của compositor, input, overlay,
  blur, clipboard và cấu hình cache.
- **Pipeline render bị nhân đôi** giữa backend Winit và Udev; logic drag/resize, xử lý
  button/touch, titlebar geometry, trích xuất app_id/title cũng bị nhân đôi.
- **Cấu hình bị đọc trong hot path** (`load_gesture_config()` được gọi mỗi lần touch/touchpad
  motion), trong khi có nhiều nguồn cấu hình và cache khác nhau.
- **GTK/GLib nằm trong core state và trong thread của compositor**, tạo rủi ro race, treo
  main context, và làm core không thể test độc lập.
- **Không có CI, không có workspace lints, không có rust-toolchain pin, không có kiểm tra
  layer dependency**. 39 chỗ `unsafe` nhưng **0 chỗ có `// SAFETY`**. 107 chỗ
  `unwrap()/expect(`, 40 chỗ `let _ =` bỏ qua lỗi.
- **Tên gọi không nhất quán** giữa `BabyDra`, `BabyDraDE`, `babydra`, `babydrade`; README và
  một số module docs đã lỗi thời.

**Định hướng đề xuất:** chuyển sang kiến trúc phân lớp một chiều, trong đó:

```text
app → backends / shell / input / rendering / compositor / platform / keymap
                     ↓
                   model
```

`compositor` chỉ còn protocol/state/space/seat; `rendering` chỉ render; `input` chỉ biến sự
kiện thiết bị thành intention; `shell` chứa overlay/UI; `platform` chứa config, metadata,
icon, process, env; `model` chứa dữ liệu thuần (plain data) dùng chung. Các quy tắc biên
được mã hóa thành **fitness function** trong `xtask` và CI, không chỉ là quy ước trong tài liệu.

Lộ trình ngắn nhất để có giá trị lớn nhất:

1. **Giai đoạn 0:** dựng guardrails (CI, lints, xtask, baseline).
2. **Giai đoạn 1:** tách các kiểu dữ liệu thuần và pipeline render dùng chung, sửa các bug
   correctness rõ ràng (`blur_all`, screenshot flag).
3. **Giai đoạn 2:** đảo phụ thuộc, tách state, đưa config/metadata ra khỏi hot path.
4. **Giai đoạn 3:** di chuyển module, tách process cho shell UI.
5. **Giai đoạn 4–5:** siết test/coverage, dọn dead code, cập nhật tài liệu.

---

## 2. Phương pháp và số liệu

### 2.1 Phương pháp

- Đọc toàn bộ 99 file Rust trong `crates/window-manager/src` (15.294 LOC).
- Chạy `cargo check -p window-manager` và `cargo clippy -p window-manager --all-targets`.
- Lập bản đồ phụ thuộc bằng cách grep các import `crate::...`, `mod`, `pub use`.
- Kiểm tra các điểm nóng: frame loop của hai backend, input pipeline, state, config, blur,
  titlebar, overlay, lock/screenshot/clipboard.
- Đối chiếu với `docs/architecture.md`, `README.md`, `Cargo.toml` của workspace và crate.
- Không chạy profiling runtime, không chạy toàn bộ test trên môi trường có GPU/DRM; các kết
  luận về hiệu năng dựa trên code path và số liệu tĩnh.

### 2.2 Số liệu hiện trạng

| Chỉ số | Giá trị |
|---|---:|
| Tổng file Rust | 99 |
| Tổng LOC | 15.294 |
| Module `#[cfg(test)]` | 22 |
| Số lần xuất hiện `unsafe` | 39 |
| Số comment `// SAFETY` | 0 |
| `unwrap()` + `expect(` | 107 |
| `let _ =` | 40 |
| Khai báo `pub` / `pub(crate)` | 386 |
| File > 300 LOC | 17 |
| File > 400 LOC | 6 |
| `cargo check` | pass |
| `cargo clippy` | 1 warning trong crate (`rendering/titlebar/color.rs:49`), thêm warning ở `services` và `ui-kit` |

**Nhận xét:** với một binary crate không có consumer bên ngoài, 386 khai báo `pub/pub(crate)`
là dấu hiệu biên module lỏng. 39 `unsafe` không có SAFETY là rủi ro audit. 107 `unwrap/expect`
trong compositor là rủi ro panic khi gặp trạng thái Wayland bất thường.

---

## 3. Kiến trúc hiện tại

### 3.1 Cây module

```text
crates/window-manager/src/
├── main.rs
├── backends/
│   ├── mod.rs
│   ├── winit.rs
│   └── udev/
│       ├── mod.rs
│       ├── gpu.rs
│       ├── render.rs
│       ├── session.rs
│       └── surface.rs
├── compositor/
│   ├── mod.rs
│   ├── state.rs
│   ├── state/
│   │   ├── active_window.rs
│   │   ├── blur.rs
│   │   ├── hit_test.rs
│   │   ├── switcher.rs
│   │   └── window/
│   │       ├── actions.rs
│   │       ├── animation.rs
│   │       ├── sizing.rs
│   │       ├── state_data.rs
│   │       └── mod.rs
│   ├── handlers/
│   │   ├── compositor.rs
│   │   ├── layer_shell.rs
│   │   ├── xdg_decoration.rs
│   │   └── xdg_shell.rs
│   ├── input/
│   │   ├── keyboard/
│   │   ├── pointer/
│   │   ├── touch/
│   │   ├── touchpad/
│   │   └── common/
│   └── cursor/
├── extensions/
│   ├── clipboard/
│   ├── launcher/
│   ├── lock/
│   ├── screenshot/
│   └── switcher/
├── keymap/
└── rendering/
    ├── animation/
    ├── blur/
    ├── elements/
    └── titlebar/
```

`main.rs` khai báo:

```rust
mod backends;
mod compositor;
mod extensions;
mod keymap;
pub mod rendering;
```

Điểm bất thường đầu tiên: `rendering` là `pub mod` trong một binary crate, còn các module
còn lại là private. Điều này cho thấy visibility chưa được thiết kế có chủ đích.

### 3.2 Luồng runtime hiện tại

```text
main::run()
  ├─ ensure_runtime_dir()
  ├─ gtk4::init()
  ├─ services::config::ensure_babydra_dirs()
  ├─ EventLoop + Display<BabydraState>
  ├─ BabydraState::new()
  ├─ clipboard channel
  ├─ backends::init()
  │    ├─ nếu có WAYLAND_DISPLAY/DISPLAY → winit::init()
  │    └─ nếu không → udev::init()
  ├─ export biến môi trường (WAYLAND_DISPLAY, XDG_*, ...)
  ├─ autostart
  └─ event_loop.run()
```

Trong mỗi frame:

```text
backend event
  → state.process_input_event()        (compositor/input)
  → state.update_window_animations()
  → state.update_clipboard()
  → dựng Vec<CustomRenderElements>:
       cursor
       layer Overlay/Top
       launcher
       switcher
       windows
       layer Bottom/Background
  → render_output() / render_frame()
  → screenshot nếu có flag
  → send_frame() cho windows + layer surfaces
  → space.refresh(), popups.cleanup(), flush_clients()
```

Luồng này đúng về ý tưởng, nhưng **được viết lại hai lần** ở `backends/winit.rs` và
`backends/udev/render.rs`, với thứ tự và xử lý screenshot khác nhau.

### 3.3 Đồ thị phụ thuộc hiện tại

```text
main ─────► backends ─────► compositor ◄──── extensions
  │            │                 ▲              │
  │            ▼                 │              ▼
  ├────────► rendering ◄─────────┘         rendering
  │            │
  │            └──────────────► compositor (window_animation_frame)
  ├────────► extensions
  └────────► keymap ─────► services
```

Các cạnh thực tế (số lượng import site):

| Từ | Đến | Số site | Ví dụ |
|---|---|---:|---|
| `compositor` | `rendering` | 13 | `state/hit_test.rs:53` gọi `rendering::titlebar::hit_test` |
| `rendering` | `compositor` | 4 | `rendering/elements/window.rs:13` import `compositor::window_animation_frame` |
| `compositor` | `extensions` | 8 | `state.rs:182` giữ `extensions::launcher::LauncherState` |
| `extensions` | `compositor` | 4 | `extensions/switcher/element.rs:23` import `compositor::AltTabSwitcher` |
| `backends` | `compositor`, `extensions`, `rendering` | nhiều | `winit.rs`, `udev/render.rs` |
| `keymap` | `services` | 1 | `keymap/config.rs` dùng `services::config::get_keymap_path` |

Hai chu trình rõ ràng:

1. **`compositor` ↔ `rendering`**
   - `rendering/elements/window.rs` cần `window_animation_frame()` từ compositor state.
   - `compositor/state/hit_test.rs` cần `rendering::titlebar` để hit-test titlebar.
2. **`compositor` ↔ `extensions`**
   - `compositor/state.rs` sở hữu `LauncherState` và `AltTabSwitcher`.
   - `extensions/switcher/element.rs` và input lại gọi ngược vào `compositor::AltTabSwitcher`.
   - `compositor/input/keyboard/shortcuts.rs` gọi trực tiếp `extensions::lock`,
     `extensions::clipboard`, `extensions::screenshot`.

Hệ quả: không thể compile/test một layer độc lập; thay đổi ở rendering có thể buộc sửa
compositor và ngược lại; khó áp dụng quy tắc "core không biết shell".

---

## 4. Điểm mạnh

Cần ghi nhận những điểm đã làm tốt để không phá vỡ khi refactor:

1. **Ý tưởng phân lớp ban đầu rõ ràng.** Các tên `backends`, `compositor`, `rendering`,
   `extensions`, `keymap` phản ánh đúng trách nhiệm ở mức cao.
2. **Hai backend thực sự hoạt động.** Winit cho nested development, Udev/DRM cho TTY.
   Đây là nền tảng tốt để tách `BackendRuntime` và `FrameComposer`.
3. **Tái sử dụng libs workspace.** `animation`, `common`, `services`, `ui-kit` đã gom được
   animation primitives, constants, config, theme, dock layout.
4. **Có ý thức test.** 22 module test, tập trung ở animation, blur, titlebar, state.
5. **Không có `TODO`/`FIXME`/`panic!`/`todo!`** trong code — ít nhất không để lại dấu vết
   chưa hoàn thiện.
6. **Tính năng phong phú:** titlebar tự vẽ, blur, launcher, switcher, lock screen, screenshot
   editor, clipboard history, gesture touch/touchpad.
7. **`cargo check` sạch**, clippy chỉ có một warning trong crate.
8. **Đã có một số abstraction đúng hướng:** `CustomRenderElements`, `BlurRenderElement`,
   `DockState`, `Animation`/`Tween`, `KeyboardAction`.

Vấn đề chính không phải là thiếu tính năng, mà là **các tính năng đang xâm lấn lẫn nhau**
và **không có hàng rào tự động** để giữ biên.

---

## 5. Phát hiện và phân tích

### 5.1 Bảng tổng hợp mức độ

| ID | Mức độ | Phát hiện | Bằng chứng chính |
|---|---|---|---|
| A1 | **Critical** | Chu trình `compositor` ↔ `rendering` | `rendering/elements/window.rs:13`, `compositor/state/hit_test.rs:53` |
| A2 | **Critical** | Chu trình `compositor` ↔ `extensions` | `compositor/state.rs:182`, `extensions/switcher/element.rs:23` |
| A3 | **Critical** | `BabydraState` là god object | `compositor/state.rs:147-201` |
| B1 | **High** | Pipeline frame bị nhân đôi giữa hai backend | `backends/winit.rs:93-230`, `backends/udev/render.rs:122-246` |
| B2 | **High** | Drag/resize bị nhân đôi pointer/touch | `pointer/drag.rs`, `touch/drag.rs` |
| B3 | **High** | Xử lý button/touch bị nhân đôi | `pointer/button.rs`, `touch/events.rs` |
| B4 | **High** | Config đọc trong hot path | `touch/gestures.rs:97,141`, `touchpad/swipe.rs:108,179` |
| B5 | **High** | GTK/GLib trong core state và thread compositor | `state/switcher.rs:328`, `extensions/lock/mod.rs:25-30` |
| B6 | **High** | Không có enforcement kiến trúc | không có CI, lints, xtask |
| C1 | **Medium** | `blur_all` bị bỏ qua | `rendering/blur/engine.rs:87-100` |
| C2 | **Medium** | Titlebar geometry bị nhân đôi | `titlebar/mod.rs`, `titlebar/hit_test.rs` |
| C3 | **Medium** | Trích xuất app_id/title lặp lại | `active_window.rs:11`, `state/switcher.rs:302`, `blur/engine.rs:170` |
| C4 | **Medium** | Screenshot flag bị tiêu thụ trong vòng lặp surface | `udev/render.rs:219` |
| C5 | **Medium** | `unsafe` không có SAFETY | 39 occurrence, 0 comment |
| C6 | **Medium** | `set_var` sau khi GTK/threads đã khởi tạo | `main.rs:77-84`, `winit.rs:65`, `udev/mod.rs:31` |
| C7 | **Medium** | Visibility/dead code/stale alias | `rendering/titlebar/input.rs`, `sample_window_color_cached`, `AltTabEntry.app_id` |
| D1 | **Low** | Clippy warning | `rendering/titlebar/color.rs:49` |
| D2 | **Low** | README/docs lỗi thời | `README.md:38-41`, `extensions/launcher/mod.rs:6` |
| D3 | **Low** | Tên gọi không nhất quán | `BabyDra`, `BabyDraDE`, `babydra`, `babydrade` |

### 5.2 God object `BabydraState`

`compositor/state.rs:147-201` chứa 38 field, trải rộng nhiều trách nhiệm:

```rust
pub(crate) struct BabydraState {
    // --- protocol / display ---
    pub(crate) start_time: Instant,
    pub(crate) socket_name: OsString,
    pub(crate) display_handle: DisplayHandle,
    pub(crate) space: Space<Window>,
    pub(crate) loop_signal: LoopSignal,
    pub(crate) compositor_state: CompositorState,
    pub(crate) xdg_shell_state: XdgShellState,
    pub(crate) _xdg_decoration_state: XdgDecorationState,
    pub(crate) wlr_layer_shell_state: WlrLayerShellState,
    pub(crate) _pointer_gestures_state: PointerGesturesState,
    pub(crate) _cursor_shape_state: CursorShapeManagerState,
    pub(crate) shm_state: ShmState,
    pub(crate) _output_manager_state: OutputManagerState,
    pub(crate) seat_state: SeatState<BabydraState>,
    pub(crate) data_device_state: DataDeviceState,
    pub(crate) popups: PopupManager,
    pub(crate) seat: Seat<BabydraState>,

    // --- clipboard ---
    pub(crate) clipboard_pending: Option<Vec<String>>,
    pub(crate) clipboard_capture: Option<Vec<String>>,
    pub(crate) clipboard_sender: Option<Sender<Option<String>>>,

    // --- input / interaction ---
    pub(crate) move_state: Option<MoveState>,
    pub(crate) resize_state: Option<ResizeState>,
    pub(crate) last_titlebar_click: Option<(Instant, Point<f64, Logical>, Window)>,
    pub(crate) trackpad_swipe: Option<TrackpadSwipeBuffer>,
    pub(crate) alt_tab_switcher: Option<AltTabSwitcher>,
    pub(crate) pointer_location: Point<f64, Logical>,
    pub(crate) cursor_visible: bool,
    pub(crate) cursor_icon: CursorIcon,
    pub(crate) pointer_pressed_buttons: HashSet<u32>,
    pub(crate) touch_slots: HashSet<TouchSlot>,
    pub(crate) overlay_touch: Option<OverlayTouchState>,
    pub(crate) touch_gesture: TouchGestureBuffer,

    // --- feature state ---
    pub(crate) launcher: LauncherState,
    pub(crate) blur_engine: Arc<Mutex<BlurEngine>>,

    // --- window list ---
    pub(crate) windows: Vec<Window>,

    // --- config cache ---
    pub(crate) touchpad_natural_scroll: bool,
    pub(crate) mouse_natural_scroll: bool,
}
```

Hệ quả:

- **Bất kỳ module nào cũng có thể mutate bất kỳ state nào.** Input có thể sửa launcher,
  blur, clipboard; rendering có thể đọc/ghi window list.
- **Borrow checker buộc phải chia nhỏ hàm hoặc dùng `take()`/`clone()`**; ví dụ
  `state.move_state.take()` trong drag, `state.alt_tab_switcher.as_ref()` lặp lại.
- **Không thể test một feature độc lập.** Muốn test launcher phải dựng cả `BabydraState`
  với display, seat, space.
- **Hai nguồn sự thật cho window order:** `space: Space<Window>` và `windows: Vec<Window>`.
  `raise_window`, `cleanup_windows`, `active_window` phải tự đồng bộ hai cấu trúc.
- **Feature state nằm trong core:** `launcher`, `alt_tab_switcher` là UI state nhưng sống
  trong compositor state.

**Đề xuất:** phân rã thành các struct nhỏ, truyền context hẹp thay vì `&mut BabydraState`.

```rust
pub struct CompositorState {
    pub display: DisplayState,
    pub globals: Globals,
    pub seat: Seat<AppState>,
    pub space: Space<Window>,
    pub popups: PopupManager,
    pub clipboard: ClipboardState,
}

pub struct WindowStore {
    windows: Vec<Window>,
    metadata: HashMap<WindowId, WindowMetadata>,
    active: Option<WindowId>,
}

pub struct InputState {
    pub pointer: PointerState,
    pub touch: TouchState,
    pub gesture: GestureState,
    pub drag: Option<DragSession>,
    pub resize: Option<ResizeSession>,
    pub last_titlebar_click: Option<TitlebarClick>,
}

pub struct RenderState {
    pub blur: BlurEngine,
    pub cursor: CursorCache,
    pub titlebar: TitlebarCache,
    pub frame_timing: FrameTiming,
}
```

`AppState` chỉ là composition root:

```rust
pub struct AppState {
    pub compositor: CompositorState,
    pub windows: WindowStore,
    pub input: InputState,
    pub render: RenderState,
    pub overlays: OverlayRegistry,
    pub config: Arc<RuntimeConfig>,
    pub platform: PlatformServices,
}
```

Quy tắc: hàm xử lý input nhận `&mut InputState`, `&mut WindowStore`, `&mut OverlayRegistry`,
`&RuntimeConfig`, không nhận `&mut AppState`.

### 5.3 Trùng lặp pipeline render

Hai frame loop gần như giống nhau:

- `backends/winit.rs:93-230`
- `backends/udev/render.rs:122-246`

Cả hai đều:

1. Cập nhật animation/clipboard.
2. Đặt cursor ở đầu danh sách element.
3. Render layer Overlay/Top.
4. Render launcher.
5. Render switcher.
6. Render windows theo thứ tự front-to-back.
7. Render layer Bottom/Background.
8. Render output.
9. Xử lý screenshot.
10. Gửi `send_frame` cho windows và layer surfaces.
11. `space.refresh()`, `popups.cleanup()`, `flush_clients()`.

Khác biệt:

| Khía cạnh | Winit | Udev |
|---|---|---|
| Output | một output | nhiều surface/output |
| Bind framebuffer | `backend.bind()` | `surface_data.compositor.render_frame()` |
| Screenshot | sau render, có restore EGL surface | trong vòng lặp surface, không restore |
| Frame scheduling | `request_redraw()` | `TimeoutAction::ToInstant(deadline + FRAME_INTERVAL)` |
| Damage | `OutputDamageTracker` | `FrameFlags::DEFAULT` |

Rủi ro:

- Sửa thứ tự element ở một backend nhưng quên backend kia.
- Screenshot ở Udev chỉ chụp surface đầu tiên vì `take_requested()` trả `true` một lần
  (`udev/render.rs:219` nằm trong `for surface_data in surfaces.iter_mut()`).
- Winit phải restore EGL surface sau capture; Udev không cần, nhưng logic này đang nằm rải
  rác trong backend thay vì trong `FrameComposer`.
- Không có test nào đảm bảo hai backend tạo cùng một danh sách element.

**Đề xuất:** tách `FrameComposer` dùng chung.

```rust
pub struct FrameContext<'a> {
    pub output: &'a Output,
    pub output_size: Size<i32, Physical>,
    pub scale: Scale<f64>,
    pub time: Instant,
    pub config: &'a RuntimeConfig,
    pub windows: &'a [WindowRenderData],
    pub overlays: &'a OverlaySet,
    pub pointer: Point<f64, Logical>,
}

pub trait FrameComposer {
    fn compose(
        &mut self,
        renderer: &mut GlowRenderer,
        ctx: &FrameContext<'_>,
    ) -> Result<Vec<CustomRenderElements>, RenderError>;
}
```

Backend chỉ còn:

```rust
let elements = composer.compose(renderer, &ctx)?;
backend.present(elements)?;
```

Screenshot trở thành một bước của `FrameComposer` hoặc `App`, không nằm trong backend.

### 5.4 Trùng lặp input: drag/resize và button/touch

#### Drag/resize

- `compositor/input/pointer/drag.rs:24-181`
- `compositor/input/touch/drag.rs:10-132`

Cả hai xử lý cùng state machine:

1. Lấy `move_state`/`resize_state`.
2. Kiểm tra threshold.
3. Nếu chưa drag, kiểm tra maximized/fullscreen và restore/unmaximize.
4. Cập nhật vị trí/kích thước.
5. Gửi configure cho client.
6. Cập nhật cursor.

Khác biệt:

- Pointer dùng threshold 4 px (`DRAG_THRESHOLD_SQ`), touch dùng 16 px².
- Pointer có timestamp, touch không.
- Touch có nhánh overlay/gesture riêng.

**Đề xuất:** một `DragSession`/`ResizeSession` trong `input` với policy threshold:

```rust
pub struct DragPolicy {
    pub threshold_sq: f64,
    pub restore_on_drag: bool,
}

pub enum PointerSource {
    Mouse { time: u32 },
    Touch,
}
```

#### Button/touch

- `compositor/input/pointer/button.rs:25-310`
- `compositor/input/touch/events.rs:27-496`

Cả hai đều:

- Kiểm tra switcher trước.
- Kiểm tra launcher.
- Kiểm tra titlebar action.
- Raise/focus window.
- Bắt đầu move/resize.
- Gửi event cho client.
- Xử lý overlay touch.

Logic titlebar/activation bị lặp; touch có thêm `OverlayTouchState`. Đây là nguồn lỗi
"pointer làm được nhưng touch không" hoặc ngược lại.

**Đề xuất:** tách `WindowActivation` và `TitlebarActionDispatcher` dùng chung; chỉ khác
`PointerEvent`/`TouchEvent` adapter.

### 5.5 Trùng lặp rendering phụ

| Chủ đề | Vị trí A | Vị trí B | Hướng xử lý |
|---|---|---|---|
| `render_pixels` (Cairo → `MemoryRenderBuffer`) | `extensions/launcher/buffer.rs` | `extensions/switcher/element.rs` | `rendering::buffer::render_to_memory_buffer` |
| Backdrop gradient | `extensions/launcher/buffer.rs::render_right_backdrop_pixels` | `extensions/switcher/element.rs` | `ui_kit::drawing` hoặc `rendering::backdrop` |
| `init_layer_window` | `extensions/lock/render.rs:15` | `extensions/screenshot/editor/layer_shell.rs:7` | `shell::gtk::init_layer_window` |
| Titlebar button geometry | `rendering/titlebar/mod.rs` | `rendering/titlebar/hit_test.rs` | `model::TitlebarLayout` |
| app_id/title extraction | `compositor/state/active_window.rs:11` | `compositor/state/switcher.rs:302`, `rendering/blur/engine.rs:170` | `WindowMetadataCache` |
| Window animation frame | `compositor/state/window/animation.rs` | `rendering/elements/window.rs` | `model::WindowVisualState` |
| Screenshot/EGL restore | `backends/winit.rs:187-202` | — | `FrameComposer`/`CaptureService` |

### 5.6 Config và metadata

#### Nhiều nguồn cấu hình

- `services::config` là loader chính, có cache theo mtime.
- `rendering/blur/config.rs` tự đọc env `BABYDRADE_BLUR_*` và ghi đè `BlurConfig`.
- `keymap/config.rs` có loader + watcher riêng (`OnceLock<RwLock<Vec<Shortcut>>>`).
- `compositor/input/pointer/config.rs` và `touchpad/config.rs` có struct `MouseConfig`/
  `TouchpadConfig` riêng, chuyển đổi từ `services::models::*`.
- `BabydraState` cache `touchpad_natural_scroll`, `mouse_natural_scroll` và chỉ refresh
  trong `reload_input_config()`.

Hệ quả: cùng một giá trị có thể đến từ ba nơi; thứ tự ưu tiên không rõ; test khó.

#### Đọc cấu hình trong hot path

```rust
// compositor/input/touch/gestures.rs
if ... && load_gesture_config().three_finger_swipe_window { ... }
...
let cfg = load_gesture_config();
```

```rust
// compositor/input/touchpad/swipe.rs
if ... && load_gesture_config().three_finger_swipe_window { ... }
...
let cfg = load_gesture_config();
```

`load_gesture_config()` thực hiện `metadata(mtime)` và clone toàn bộ config. Với touch/touchpad
motion ở tần số cao, đây là I/O + allocation không cần thiết.

#### Metadata trích xuất lặp lại

Cùng một logic `with_states(... XdgToplevelSurfaceData ... app_id/title ...)` xuất hiện ở:

- `compositor/state/active_window.rs:11`
- `compositor/state/switcher.rs:302`
- `rendering/blur/engine.rs:170`

Mỗi nơi có cách xử lý lỗi/khóa khác nhau (`unwrap()` ở active_window, `ok()` ở blur).

**Đề xuất:** `WindowMetadataCache` cập nhật tại các điểm commit/rename:

```rust
pub struct WindowMetadata {
    pub id: WindowId,
    pub app_id: String,
    pub title: String,
    pub app_name: String,
    pub pid: Option<u32>,
    pub icon: Option<IconHandle>,
}

pub struct WindowMetadataCache {
    map: HashMap<WindowId, WindowMetadata>,
}

impl WindowMetadataCache {
    pub fn refresh(&mut self, window: &Window);
    pub fn remove(&mut self, id: WindowId);
    pub fn get(&self, id: WindowId) -> Option<&WindowMetadata>;
}
```

Không còn `with_states` trong render/blur/switcher.

#### `blur_all` không hoạt động

`BlurEngine::matches_window` (`rendering/blur/engine.rs:87-100`) chỉ kiểm tra
`params.enabled` và rules; **không kiểm tra `params.blur_all`**. `set_blur_all()` và env
`BABYDRADE_BLUR_ALL` vì vậy không có tác dụng lên cửa sổ.

Ngoài ra:

- `matches_app_or_title` lowercases `app_id`/`title` mỗi lần gọi → allocation mỗi frame.
- `rules_lower` chỉ được sync khi độ dài thay đổi; nếu rules đổi nhưng cùng độ dài thì cache
  stale.
- `matches_layer` loại trừ topbar/desktop/wallpaper bằng substring; dễ sai nếu namespace đổi.

**Đề xuất:** thêm nhánh `if self.params.blur_all { return true; }` (sau `enabled`), viết test
cho `blur_all = true`, và làm cho `rules_lower` luôn derive từ `app_rules` (private field,
không cho mutate trực tiếp).

### 5.7 Visibility, dead code, stale API

- `main.rs:5` khai báo `pub mod rendering;` trong binary crate. Không có external consumer.
- `rendering/titlebar/mod.rs:30-31`:
  ```rust
  // Keep old public name as an alias for external callers / tests
  pub use get_or_sample_color as sample_window_color_cached;
  ```
  Không có "external callers" trong binary crate; alias này là dead code.
- `rendering/titlebar/input.rs` chỉ re-export `hit_test, Action`; không được dùng.
- `compositor/state/switcher.rs:18` có `#[allow(dead_code)]` cho `AltTabEntry.app_id`.
- `compositor/state/window/mod.rs` có hai `#[allow(unused_imports)]` che các re-export chết.
- 386 khai báo `pub/pub(crate)` cho thấy mặc định "public" thay vì "private".

**Đề xuất:**

- Trong binary crate, mặc định private; chỉ `pub(crate)` khi thực sự cross-module.
- Bật `#![warn(unreachable_pub)]`.
- Xóa alias dead code; nếu cần compatibility, dùng `#[deprecated]` có thời hạn.
- Thay `#[allow(dead_code)]` bằng `#[expect(dead_code, reason = "...")]` và xóa khi hết lý do.
- Không dùng `#[allow(unused_imports)]`; sửa import hoặc xóa.

### 5.8 Unsafe và xử lý lỗi

#### Unsafe

39 occurrence `unsafe`, 0 `// SAFETY`:

| File | Số occurrence | Loại |
|---|---:|---|
| `extensions/lock/auth.rs` | 9 | PAM FFI, `unsafe impl Send/Sync`, callback |
| `rendering/blur/gl/pipeline.rs` | 8 | GL calls |
| `rendering/blur/gl/shaders.rs` | 6 | GL shader compile/link |
| `rendering/blur/engine.rs` | 5 | GL cache release |
| `extensions/screenshot/capture.rs` | 3 | EGL surface pointer |
| `rendering/titlebar/color.rs` | 2 | GL/EGL |
| `rendering/blur/gl/gl_state.rs` | 2 | GL state |
| `backends/udev/gpu.rs` | 2 | DRM/GBM |
| `compositor/state.rs` | 1 | `Display`/state |
| `backends/winit.rs` | 1 | EGL surface restore |

Rủi ro: không có tài liệu về invariant; khó audit; dễ undefined behavior khi refactor.

**Đề xuất:**

- Mỗi `unsafe` block có `// SAFETY: ...` giải thích invariant.
- Cô lập unsafe vào `platform::ffi` (PAM), `rendering::blur::gl` (GL), `platform::drm` (DRM/GBM).
- Cung cấp safe wrapper; không cho module khác gọi trực tiếp.
- Bật `#![deny(unsafe_op_in_unsafe_fn)]` và `clippy::undocumented_unsafe_blocks`.
- Chạy `cargo geiger` để theo dõi; cân nhắc `cargo miri test` cho phần thuần.

#### `unwrap/expect` và `let _ =`

- 107 `unwrap()/expect(`; nhiều ở `state/window/animation.rs` (13), `state/window/sizing.rs` (7),
  `pointer/button.rs` (5), `state.rs` (6).
- 40 `let _ =`; nhiều chỗ bỏ qua lỗi ghi file, gửi clipboard, kill process.

**Đề xuất:**

- Trong code không phải test: không `unwrap()`; dùng `?` với error type hoặc `expect("invariant: ...")`
  có ghi chú.
- `let _ =` chỉ dùng cho cleanup không quan trọng; còn lại log `warn!`.
- Thêm `thiserror` cho lỗi typed ở biên; `anyhow` ở `app`.
- Bật `clippy::unwrap_used`/`expect_used` ở mức `warn`, tiến tới `deny` sau khi dọn.

### 5.9 Process/threading và môi trường

#### GTK trong compositor

- `main.rs:36` gọi `gtk4::init()` trước khi tạo event loop.
- `compositor/state/switcher.rs:328` gọi `ui_kit::get_app_icon_pixbuf_any` trong
  `std::panic::catch_unwind` — GTK/GIO nằm trong core state.
- `extensions/lock/mod.rs:25-30` spawn thread `babydra-lock-screen` rồi chạy GTK app trong
  thread đó.
- `extensions/clipboard/mod.rs:3-4` nói rằng GTK không thể chia sẻ default GLib main context,
  nên clipboard chạy subprocess. Điều này **mâu thuẫn** với lock screen chạy GTK trong thread
  của compositor.

Hệ quả:

- GTK main context và calloop event loop có thể tranh chấp.
- `gtk4::init()` được gọi trước khi `set_var` môi trường; env thay đổi sau khi thread/GTK
  khởi tạo có thể không được thấy hoặc gây data race.
- Lock screen dùng `std::env::args()` của compositor, nên subcommand/args dễ bị lẫn.
- Screenshot editor và clipboard picker chạy cùng executable với subcommand; khó test,
  khó đóng gói.

#### Biến môi trường

`std::env::set_var` xuất hiện ở:

- `main.rs:29` (`XDG_RUNTIME_DIR` fallback)
- `main.rs:77-84` (`WAYLAND_DISPLAY`, `XDG_*`, toolkit hints)
- `backends/winit.rs:65` (`WAYLAND_DISPLAY`)
- `backends/udev/mod.rs:31` (`WAYLAND_DISPLAY`)

Trong Rust 2024, `set_var` là `unsafe` vì có thể race với thread khác. Ngay cả edition 2021,
gọi sau `gtk4::init()`/spawn thread là không an toàn về mặt logic.

**Đề xuất:**

- Tập trung toàn bộ env vào `app::bootstrap` **trước** `gtk4::init()` và trước khi spawn thread.
- Dùng `Command::env` cho process con; không dựa vào env toàn cục.
- Bỏ `set_var` khỏi backend; backend nhận `socket_name`/`WaylandDisplay` qua config.
- Nếu buộc phải set sau init, ghi rõ lý do và dùng cơ chế an toàn (khởi tạo trước).

#### Lock screen và `grim`

`extensions/lock/render.rs` dùng `grim` để chụp transition image. Đây là dependency ngoài
không được khai báo trong README/Cargo. Nếu `grim` không có, lock transition có thể fail âm thầm.

**Đề xuất:** thay bằng capture nội bộ (đã có `screenshot::capture`) hoặc cô lập `grim` trong
`platform::process` với kiểm tra availability và thông báo lỗi rõ ràng.

### 5.10 Test, CI và tooling

Hiện có:

- 22 module `#[cfg(test)]`.
- Không có thư mục `tests/` integration test.
- Không có CI config (`.github/workflows` không tồn tại).
- Không có `rust-toolchain.toml`.
- Không có `[workspace.lints]`.
- Không có `deny.toml`, `clippy.toml`, `nextest.toml`, `typos.toml`.
- Không có crate-level `#![deny]`/`#![warn]`.
- Không có coverage gate.

Hệ quả: các quy tắc kiến trúc trong tài liệu sẽ bị vi phạm dần; không ai bị chặn khi thêm
import sai layer hoặc `unwrap()` mới.

### 5.11 Naming và tài liệu

#### Naming

Các biến thể đang tồn tại:

| Ngữ cảnh | Giá trị |
|---|---|
| Tên sản phẩm | `BabyDra`, `BabyDraDE` |
| Crate/binary | `babydra-window-manager` |
| Config dir | `~/.babydra` |
| Runtime dir fallback | `babydrade-runtime-*` |
| Seat | `babydrade` |
| Output | `babydrade-0` |
| App ID | `org.babydra.*` |
| Namespace layer-shell | `babydra-lock`, `babydra-screenshot` |
| `XDG_CURRENT_DESKTOP` | `BabyDra` |
| Env prefix | `BABYDRADE_*` |

Không có nguồn chân lý duy nhất. Điều này ảnh hưởng đến desktop integration, icon lookup,
session detection, và tài liệu.

**Đề xuất:** định nghĩa `platform::brand`:

```rust
pub const DISPLAY_NAME: &str = "BabyDraDE";
pub const ID: &str = "babydra";
pub const APP_ID_PREFIX: &str = "org.babydra";
pub const CONFIG_DIR: &str = ".config/babydra";
pub const ENV_PREFIX: &str = "BABYDRADE";
```

Chọn một quy ước:

- Brand hiển thị: `BabyDraDE`.
- Package/binary/prefix: `babydra`.
- App ID: `org.babydra.*`.
- Config: XDG `~/.config/babydra` (có migration từ `~/.babydra`).
- `XDG_CURRENT_DESKTOP=BabyDraDE` (có thể thêm alias `BabyDra` trong giai đoạn chuyển tiếp).
- Seat: `seat0`; output ảo: `babydra-0`.

#### Tài liệu

- `docs/architecture.md` chỉ 20 dòng, không có module dependency rules, không có state ownership.
- `README.md:38-41` nói "no custom decorations, app menu, XWayland, or direct DRM/TTY backend",
  trái với code hiện tại (có titlebar tự vẽ và backend Udev/DRM).
- `extensions/launcher/mod.rs:6` nhắc module `layout` không tồn tại.
- `docs/animations-constants-enums.md` tốt nhưng chỉ bao phủ animation/constants/dock.

**Đề xuất:** cập nhật README, viết lại `docs/architecture.md` kèm dependency matrix, thêm
`docs/adr/` cho các quyết định lớn, và thêm module docs `//!` mô tả trách nhiệm + layer được
phép import.

### 5.12 Các smell correctness khác

- **`winit` transform mặc định `Flipped180`** (`backends/winit.rs:55-60`). Cần xác minh đây
  có phải chủ ý hay là lỗi; backend nested thông thường dùng `Transform::Normal`.
- **Hai nguồn window order** (`space` và `windows`) như đã nêu.
- **`active_window()` phụ thuộc thứ tự `windows`**, trong khi render dùng `space.elements().rev()`;
  nếu hai cấu trúc lệch nhau, focus và render có thể khác.
- **`window_under` kiểm tra titlebar trước `space.element_under`**, với logic occlusion riêng;
  cần test cho cửa sổ chồng nhau, maximized, fullscreen.
- **`last_titlebar_click` lưu `Window`** thay vì `WindowId`; nếu window chết, state vẫn giữ.
- **`catch_unwind` cho icon GTK** che panic; nên xử lý lỗi tường minh hoặc tách process.
- **Screenshot temp path cố định** `/tmp/babydra-screenshot-copy.png`
  (`extensions/screenshot/editor/handle/clipboard.rs:14`) — nên dùng `XDG_RUNTIME_DIR` +
  file unique/permissions.
- **`ensure_runtime_dir` fallback** tạo thư mục trong `/tmp` với UID; cần kiểm tra quyền sở hữu
  và tránh symlink attack.
- **`BlurEngine::matches_app_or_title` lowercases mỗi frame**; có thể cache lowercase metadata.

---

## 6. Kiến trúc đích

### 6.1 Nguyên tắc thiết kế

1. **Một chiều phụ thuộc.** Không có chu trình giữa các layer.
2. **Core không biết UI.** `compositor` không import `rendering`, `shell`, `gtk4`, `ui-kit`.
3. **Rendering không biết protocol state.** `rendering` nhận `WindowVisualState`/`FrameContext`
   từ `model`, không gọi `compositor`.
4. **Input không biết rendering/shell.** Input tạo `Action`/`Intent`; `app`/`compositor` thực thi.
5. **Shell là adapter.** Launcher/switcher/lock/screenshot/clipboard implement trait/overlay
   và được `app` đăng ký; core không gọi trực tiếp.
6. **Config là snapshot bất biến.** Không đọc file/env trong hot path.
7. **Metadata là cache.** Trích xuất app_id/title/icon một lần tại commit; consumer đọc cache.
8. **Biên được enforce tự động.** Mọi quy tắc phải có kiểm tra trong CI.

### 6.2 Cấu trúc top-level đề xuất

```text
crates/window-manager/src/
├── main.rs                 # entry point cực mỏng
├── app/                    # composition root
│   ├── mod.rs
│   ├── bootstrap.rs        # env, logging, config, GTK init (nếu cần)
│   ├── runtime.rs          # event loop, backend selection, autostart
│   └── commands.rs         # subcommand dispatch
├── compositor/             # protocol/state/space/seat, không render/GTK
│   ├── mod.rs
│   ├── state/
│   ├── handlers/
│   ├── focus/
│   └── ports.rs            # Overlay, ActionSink, WindowOps traits
├── input/                  # thiết bị → intention
│   ├── mod.rs
│   ├── keyboard/
│   ├── pointer/
│   ├── touch/
│   ├── touchpad/
│   └── gesture/
├── rendering/              # frame composition, elements, effects
│   ├── mod.rs
│   ├── frame.rs            # FrameComposer
│   ├── elements/
│   ├── titlebar/
│   ├── blur/
│   └── animation/
├── shell/                  # overlay/UI features
│   ├── mod.rs
│   ├── launcher/
│   ├── switcher/
│   ├── lock/
│   ├── screenshot/
│   └── clipboard/
├── platform/               # OS integration
│   ├── mod.rs
│   ├── config.rs           # ConfigService + watcher
│   ├── metadata.rs         # WindowMetadataCache
│   ├── icons.rs            # IconProvider (GTK/GIO)
│   ├── processes.rs        # spawn/exec, env
│   ├── paths.rs            # XDG dirs, brand constants
│   └── gtk.rs              # GTK helpers (chỉ shell dùng)
├── keymap/                 # shortcut config + matcher
│   ├── mod.rs
│   ├── config.rs
│   ├── mapping.rs
│   └── matcher.rs
├── backends/               # adapter Winit/Udev
│   ├── mod.rs
│   ├── winit.rs
│   └── udev/
└── model/                  # plain data dùng chung (mới)
    ├── mod.rs
    ├── ids.rs
    ├── window.rs           # WindowId, WindowMetadata, WindowVisualState
    ├── titlebar.rs         # TitlebarLayout, TitlebarAction
    ├── config.rs           # RuntimeConfig + section types
    └── input.rs            # PointerEvent, TouchEvent, InputOutcome
```

**Ghi chú:** `model` là leaf module, chỉ phụ thuộc std + các libs thuần (`common`, `animation`).
Nếu muốn giữ đúng danh sách ban đầu (`app`, `compositor`, `input`, `rendering`, `shell`,
`platform`, `keymap`), có thể đặt `model` dưới `compositor::model`; nhưng khi đó `rendering`
sẽ phải import `compositor::model`, tạo cảm giác phụ thuộc ngược. Top-level `model` sạch hơn.

### 6.3 Ma trận phụ thuộc mục tiêu

Ký hiệu: ✅ được phép, ❌ cấm.

| From \ To | model | platform | compositor | input | rendering | shell | keymap | backends | app |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `model` | — | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| `platform` | ✅ | — | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| `compositor` | ✅ | ❌ | — | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| `keymap` | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | — | ❌ | ❌ |
| `input` | ✅ | ✅ | ✅ | — | ❌ | ❌ | ✅ | ❌ | ❌ |
| `rendering` | ✅ | ✅ | ❌ | ❌ | — | ❌ | ❌ | ❌ | ❌ |
| `shell` | ✅ | ✅ | ✅ | ❌ | ✅ | — | ❌ | ❌ | ❌ |
| `backends` | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | — | ❌ |
| `app` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | — |

Giải thích các cạnh quan trọng:

- `input → compositor`: input cần đọc/ghi state qua các API hẹp (`WindowOps`, `FocusOps`,
  `OverlayRegistry`), không import `shell`/`rendering`.
- `shell → compositor`: shell implement `Overlay`/`ActionSink`; chỉ dùng trait/port, không
  truy cập field trực tiếp.
- `rendering → platform`: chỉ để lấy theme/asset snapshot; nếu muốn tuyệt đối sạch, đưa
  `ThemeSnapshot` vào `model` và bỏ cạnh này.
- `backends → rendering`: backend gọi `FrameComposer`; không tự dựng element list.
- `compositor → platform`: **không**; config/metadata do `app` inject vào compositor dưới
  dạng `RuntimeConfig`/`WindowMetadataCache` (thuộc `model`/`compositor`).
- `shell → input`: **không**; overlay nhận event qua trait từ `compositor`/`app`.

### 6.4 Interface chính

#### `FrameComposer`

```rust
pub struct FrameContext<'a> {
    pub output: &'a Output,
    pub output_size: Size<i32, Physical>,
    pub scale: Scale<f64>,
    pub time: Instant,
    pub config: &'a RuntimeConfig,
    pub windows: &'a [WindowRenderData],
    pub overlays: &'a OverlaySet,
    pub pointer: Point<f64, Logical>,
    pub cursor_visible: bool,
}

pub trait FrameComposer {
    fn compose(
        &mut self,
        renderer: &mut GlowRenderer,
        ctx: &FrameContext<'_>,
    ) -> Result<Vec<CustomRenderElements>, RenderError>;
}
```

`WindowRenderData` gồm `Window`, `location`, `WindowVisualState`, `WindowMetadata`; không
gọi `with_states` trong lúc compose.

#### `InteractionController`

```rust
pub enum InputOutcome {
    Ignored,
    Consumed,
    Action(Action),
}

pub trait InteractionController {
    fn pointer_button(&mut self, ctx: &mut InputContext<'_>, ev: &PointerButton) -> InputOutcome;
    fn pointer_motion(&mut self, ctx: &mut InputContext<'_>, ev: &PointerMotion) -> InputOutcome;
    fn touch(&mut self, ctx: &mut InputContext<'_>, ev: &TouchEvent) -> InputOutcome;
    fn key(&mut self, ctx: &mut InputContext<'_>, ev: &KeyEvent) -> InputOutcome;
}
```

`InputContext` gồm các con trỏ hẹp:

```rust
pub struct InputContext<'a> {
    pub input: &'a mut InputState,
    pub windows: &'a mut WindowStore,
    pub overlays: &'a mut OverlayRegistry,
    pub config: &'a RuntimeConfig,
    pub seat: &'a Seat<AppState>,
}
```

#### `RuntimeConfig`

```rust
pub struct RuntimeConfig {
    pub blur: BlurConfig,
    pub gestures: GestureConfig,
    pub mouse: MouseConfig,
    pub touchpad: TouchpadConfig,
    pub keymap: KeymapSnapshot,
    pub theme: ThemeSnapshot,
    pub env: EnvironmentConfig,
}

pub trait ConfigStore {
    fn current(&self) -> Arc<RuntimeConfig>;
    fn reload(&self) -> Result<(), ConfigError>;
}
```

Triển khai bằng `arc_swap::ArcSwap<RuntimeConfig>` hoặc `RwLock<Arc<RuntimeConfig>>`; watcher
chỉ swap ở ranh giới frame/event, không swap giữa lúc render.

#### `WindowMetadata` và `WindowMetadataCache`

```rust
pub struct WindowId(u64);

pub struct WindowMetadata {
    pub id: WindowId,
    pub app_id: String,
    pub title: String,
    pub app_name: String,
    pub pid: Option<u32>,
    pub icon: Option<IconHandle>,
}

pub struct WindowMetadataCache {
    map: HashMap<WindowId, WindowMetadata>,
}
```

Cập nhật tại `new_toplevel`, `commit`, `app_id_changed`, `title_changed`, `destroy`.
Consumer: blur, switcher, active window, titlebar, taskbar.

#### `TitlebarLayout`

```rust
pub struct TitlebarLayout {
    pub height: i32,
    pub radius: i32,
    pub buttons: [TitlebarButton; 3],
}

impl TitlebarLayout {
    pub fn compute(
        metadata: &WindowMetadata,
        geometry: Rectangle<i32, Logical>,
        maximized: bool,
        fullscreen: bool,
    ) -> Self;

    pub fn hit_test(&self, point: Point<f64, Logical>) -> Option<TitlebarAction>;
}
```

Cả `rendering::titlebar` và `compositor::hit_test` dùng cùng một layout; không hardcode
button geometry ở hai nơi.

#### `Overlay` registry

```rust
pub trait Overlay {
    fn id(&self) -> OverlayId;
    fn is_visible(&self) -> bool;
    fn layout(&self, ctx: &OverlayContext) -> Option<OverlayLayout>;
    fn render(
        &mut self,
        ctx: &mut OverlayRenderContext<'_>,
    ) -> Result<Vec<CustomRenderElements>, RenderError>;
    fn pointer(&mut self, ev: &PointerEvent) -> OverlayResponse;
    fn touch(&mut self, ev: &TouchEvent) -> OverlayResponse;
    fn key(&mut self, ev: &KeyEvent) -> OverlayResponse;
}

pub enum OverlayResponse {
    Ignored,
    Consumed,
    Action(Action),
}
```

`OverlayRegistry` nằm trong `compositor` hoặc `app`; `shell` đăng ký implementation. Input
chỉ tương tác với registry, không import `launcher`/`switcher`.

#### `BackendRuntime`

```rust
pub trait BackendRuntime {
    fn init(event_loop: &mut EventLoop<CalloopData>, app: &mut App) -> Result<Self, BackendError>
    where
        Self: Sized;

    fn run(&mut self, app: &mut App) -> Result<(), BackendError>;
}
```

Backend chỉ cung cấp renderer/surface/event source; frame composition gọi `FrameComposer`.

### 6.5 Phân rã state chi tiết

| State | Trách nhiệm | Không được chứa |
|---|---|---|
| `CompositorState` | Display, socket, globals, seat, space, popups, clipboard selection | launcher/switcher, blur engine, GTK |
| `WindowStore` | windows, z-order, active, metadata cache, animation state | input session, overlay state |
| `InputState` | pointer, touch, gesture, drag/resize session, last titlebar click | window list, rendering |
| `RenderState` | blur engine, cursor cache, titlebar cache, frame timing | protocol state, GTK |
| `OverlayState` (shell) | launcher/switcher visibility, selection, layout cache | protocol globals |
| `RuntimeConfig` | snapshot bất biến của tất cả config | file handles, mutable caches |
| `PlatformServices` | config loader/watcher, icon provider, process spawner, paths | compositor state |

Quy tắc truy cập:

- `compositor` chỉ nhận `&mut CompositorState` + `&mut WindowStore`.
- `input` nhận `&mut InputState` + `&mut WindowStore` + `&mut OverlayRegistry` + `&RuntimeConfig`.
- `rendering` nhận `&FrameContext` (immutable).
- `shell` nhận `&mut OverlayState` + `&WindowMetadataCache` + `&RuntimeConfig`; trả về
  `OverlayResponse`.
- Không có hàm nào nhận `&mut AppState` ngoại trừ `app` composition root.

### 6.6 Config và metadata tập trung

#### ConfigService

```text
platform::config::ConfigService
  ├─ load_all() -> RuntimeConfig
  ├─ watch() -> ConfigEvent
  ├─ current() -> Arc<RuntimeConfig>
  └─ reload() -> Result<(), ConfigError>
```

- Một loader duy nhất cho `babydra.conf`, `keymap.toml`, env overrides.
- `services::config` vẫn là parser/file layer; `platform` map sang `RuntimeConfig`.
- Watcher dùng calloop timer/inotify; reload tối đa 1 lần/giây; swap ở frame boundary.
- Không gọi `load_*` trong `input`, `rendering`, `compositor`.
- Env overrides được parse một lần trong `platform::config::env_overrides()`.

#### Metadata

- `WindowMetadataCache` cập nhật tại handler commit.
- Blur/switcher/active-window/titlebar đọc cache.
- Icon resolution là `IconProvider` port; implementation GTK nằm ở `platform::icons` hoặc
  `shell`, không nằm trong `compositor::state`.

#### Titlebar

- `TitlebarLayout` là nguồn duy nhất cho geometry/hit-test.
- `rendering::titlebar` chỉ vẽ từ layout; `compositor::hit_test` chỉ hit-test layout.
- Test chung cho cả hai.

### 6.7 Process/threading model

#### Mục tiêu

Compositor core là **single-threaded calloop**, không GTK, không PAM, không đọc file trong
frame loop.

```text
babydra-window-manager (Smithay/calloop)
  ├─ Wayland protocol + input + render
  ├─ overlay registry (dữ liệu thuần)
  └─ IPC với shell (nếu tách process)

babydra-shell (GTK4)
  ├─ launcher
  ├─ switcher UI (nếu cần GTK)
  ├─ lock screen + PAM
  ├─ screenshot editor
  └─ clipboard picker
```

#### Lộ trình process

- **Bước 1 (transitional):** giữ GTK trong process nhưng cô lập trong `shell` module, chạy
  trên thread riêng với GLib main context riêng; giao tiếp qua channel; không truy cập
  `BabydraState` trực tiếp.
- **Bước 2 (target):** tách `babydra-shell` thành crate/binary riêng; compositor giao tiếp
  qua Wayland layer-shell + Unix socket/D-Bus. Lock screen dùng `ext-session-lock-v1`; PAM
  nằm trong shell process.
- **Bước 3:** screenshot editor và clipboard picker là subcommand của `babydra-shell` hoặc
  binary riêng; compositor chỉ cung cấp capture buffer/path.

#### Env

- Tất cả env được set trong `app::bootstrap` trước `gtk4::init()` và trước thread.
- Backend nhận `socket_name` qua tham số, không `set_var`.
- Process con nhận env qua `Command::env`.
- Không dùng `std::env::set_var` trong `input`/`rendering`/`compositor`.

#### Lock/screenshot

- Lock: process riêng, PAM riêng, không dùng `std::env::args()` của compositor.
- Screenshot: compositor capture vào shared memory/fd; editor process đọc; không phụ thuộc
  `grim`.
- Clipboard: giữ subprocess (đã đúng hướng), nhưng chuyển sang `babydra-shell --clipboard`.

---

## 7. Lộ trình refactor

Ước lượng cho 1 kỹ sư quen codebase; có thể song song hóa một số việc.

### Giai đoạn 0 — Guardrails (1–2 ngày)

**Mục tiêu:** biến quy tắc thành kiểm tra tự động trước khi sửa kiến trúc.

Việc cần làm:

1. Thêm `rust-toolchain.toml` pin stable + `rustfmt`, `clippy`, `llvm-tools`.
2. Thêm `[workspace.lints]` trong `Cargo.toml`.
3. Thêm `clippy.toml`, `deny.toml`, `nextest.toml`, `typos.toml`.
4. Thêm `.github/workflows/ci.yml`:
   - `cargo fmt --all --check`
   - `cargo clippy --workspace --all-targets -- -D warnings`
   - `cargo test --workspace`
   - `cargo deny check`
   - `typos`
   - `cargo xtask check-layering`
5. Thêm `xtask` với các lệnh:
   - `check-layering`: đọc ma trận 6.3, parse `use crate::...`, fail nếu cạnh cấm.
   - `check-visibility`: fail nếu binary crate có `pub` ngoài allowlist.
   - `check-hot-path`: fail nếu `input/`, `rendering/`, `compositor/` chứa
     `load_*config`, `std::fs`, `std::env::var`, `gtk4::`, `Command::new` (trừ allowlist).
   - `check-unsafe`: fail nếu `unsafe` block không có `// SAFETY:`.
6. Ghi baseline số warning/test vào `docs/architecture-baseline.md`; CI chỉ fail trên vi phạm
   mới trong giai đoạn đầu, sau đó ratchet.

**Definition of Done:** CI chạy trên PR; `xtask` chạy local; baseline được ghi lại.

### Giai đoạn 1 — Tách kiểu thuần và pipeline dùng chung (3–5 ngày)

**Mục tiêu:** giảm trùng lặp và sửa bug rõ ràng mà không đổi hành vi.

Việc cần làm:

1. Tạo `model/`:
   - `WindowId`, `WindowMetadata`, `WindowVisualState`.
   - `TitlebarLayout`, `TitlebarAction`.
   - `RuntimeConfig` (chỉ types, chưa có loader).
2. Tách `TitlebarLayout` khỏi `rendering/titlebar/mod.rs` + `hit_test.rs`; dùng ở cả render
   và hit-test; thêm test geometry.
3. Tách `FrameComposer` khỏi hai backend:
   - `rendering/frame.rs` dựng element list.
   - Winit/Udev chỉ bind/present/schedule.
   - Screenshot xử lý một lần/frame, ngoài vòng lặp surface.
4. Tách `render_to_memory_buffer` dùng chung cho launcher/switcher; tách backdrop gradient.
5. Tách `init_layer_window` vào `platform::gtk` (hoặc `shell::gtk`).
6. Sửa `blur_all` + test.
7. Sửa screenshot flag ở Udev: `take_requested()` một lần trước vòng lặp; capture output
   được chọn hoặc tất cả output rõ ràng.
8. Thêm characterization test cho thứ tự element và titlebar hit-test.

**Definition of Done:** hai backend dùng cùng `FrameComposer`; không còn code dựng element
list trong backend; test mới pass; không thay đổi hành vi người dùng.

### Giai đoạn 2 — Đảo phụ thuộc và tách state (7–10 ngày)

**Mục tiêu:** phá chu trình, giảm god object, đưa config/metadata ra khỏi hot path.

Việc cần làm:

1. Định nghĩa `Overlay`/`OverlayRegistry`/`ActionSink` trong `compositor::ports`.
2. Chuyển `LauncherState` và `AltTabSwitcher` khỏi `BabydraState` sang `shell`; đăng ký
   overlay trong `app`.
3. Thêm `WindowMetadataCache`; thay các chỗ `with_states` trong active_window/switcher/blur.
4. Thêm `ConfigService` + `ArcSwap<RuntimeConfig>`; xóa `load_gesture_config()` khỏi
   gesture/swipe hot path.
5. Thêm `IconProvider` port; chuyển GTK icon resolution khỏi `state/switcher.rs`.
6. Tách `InputState`, `WindowStore`, `RenderState`; thay `&mut BabydraState` bằng context hẹp.
7. Xóa import `crate::extensions` khỏi `compositor`; xóa import `crate::compositor` khỏi
   `rendering` (dùng `model`).
8. Chuyển `window_animation_frame` thành `WindowVisualState` trong `model`, do
   `WindowStore` cung cấp.

**Definition of Done:** `xtask check-layering` pass với ma trận mục tiêu; `BabydraState`
< 150 LOC và không còn field launcher/switcher/blur; không còn `load_*config` trong hot path;
test pass.

### Giai đoạn 3 — Di chuyển module và tách process (7–10 ngày)

**Mục tiêu:** cấu trúc top-level đúng, core không GTK.

Việc cần làm:

1. Tạo `app/`, `input/`, `shell/`, `platform/`; di chuyển file theo cây 6.2.
2. `app::bootstrap` chịu trách nhiệm env, logging, config, backend selection, autostart,
   subcommand.
3. Tách `babydra-shell` (crate/binary) hoặc tối thiểu chạy GTK trong thread riêng có
   context riêng, giao tiếp qua channel.
4. Chuyển lock screen sang process riêng + `ext-session-lock-v1`; PAM trong shell process.
5. Chuyển screenshot editor/clipboard sang `babydra-shell` subcommand.
6. Loại bỏ `std::env::set_var` khỏi backend; set env trước GTK/thread.
7. Nếu chưa tách crate, thêm feature `shell-gtk` để core có thể build không GTK.

**Definition of Done:** `compositor` không import `gtk4`/`ui-kit`/`shell`; `rendering` không
import `compositor`; `app` là nơi duy nhất biết tất cả layer; process model được ghi trong
`docs/architecture.md`.

### Giai đoạn 4 — Siết test và enforcement (ongoing)

**Mục tiêu:** giữ kiến trúc không bị thoái hóa.

Việc cần làm:

1. `cargo nextest` + coverage (`cargo llvm-cov`), đặt ngưỡng:
   - `model`: ≥ 80%
   - `input`/`rendering` logic thuần: ≥ 60%
   - toàn crate: ≥ 50%, tăng dần.
2. Integration test cho state transitions: map/unmap, focus, raise, maximize, minimize,
   fullscreen, drag/resize, overlay open/close.
3. Golden/snapshot test cho titlebar layout, dock layout, frame element order.
4. Property test cho geometry/hit-test (proptest).
5. Smoke test matrix thủ công/tự động:
   - Winit nested: launch, focus, drag, resize, launcher, switcher, screenshot.
   - Udev/DRM: VT switch, hotplug output, touch, touchpad gesture.
   - Lock: PAM success/fail, multi-monitor.
   - Clipboard: copy/paste, history, source app closes.
6. CI chặn PR nếu coverage giảm hoặc layering vi phạm.

### Giai đoạn 5 — Dọn dẹp và tài liệu (3–5 ngày)

1. Xóa dead code/alias: `rendering/titlebar/input.rs`, `sample_window_color_cached`,
   `AltTabEntry.app_id`, `#[allow(unused_imports)]`.
2. Sửa naming theo `platform::brand`; thêm migration config `~/.babydra` → `~/.config/babydra`
   nếu chọn XDG.
3. Cập nhật README (bỏ mô tả sai), `docs/architecture.md` (dependency matrix + state ownership),
   thêm ADR.
4. Thêm module docs `//!` với "Allowed dependencies" và invariant.
5. Chạy `cargo doc`, `typos`, `cargo machete`, `cargo deny`; dọn warning còn lại.

---

## 8. Chuẩn enforce nghiêm ngặt

> **Nguyên tắc:** quy ước chỉ có giá trị khi được mã hóa thành kiểm tra tự động. Mọi mục
> dưới đây phải có lệnh/CI tương ứng.

### 8.1 Lints

`Cargo.toml` workspace:

```toml
[workspace.lints.rust]
unsafe_op_in_unsafe_fn = "deny"
unused_must_use = "deny"
unreachable_pub = "warn"
missing_debug_implementations = "warn"
rust_2018_idioms = "deny"

[workspace.lints.clippy]
all = "deny"
pedantic = "warn"
nursery = "warn"
unwrap_used = "warn"
expect_used = "warn"
undocumented_unsafe_blocks = "deny"
too_many_arguments = "warn"
type_complexity = "warn"
```

Crate-level:

```rust
#![deny(unsafe_op_in_unsafe_fn)]
#![deny(clippy::undocumented_unsafe_blocks)]
#![warn(clippy::unwrap_used, clippy::expect_used)]
#![warn(unreachable_pub)]
```

Ghi chú:

- `unwrap_used`/`expect_used` để `warn` trước, `deny` sau khi dọn xong; test được
  `#[cfg_attr(test, allow(...))]`.
- `pedantic`/`nursery` để `warn`, không `deny` toàn bộ để tránh noise; CI chỉ `-D warnings`
  cho `all`.
- `missing_docs` có thể bật cho `model`/`platform` sau khi có docs.

### 8.2 Visibility

- Binary crate: mặc định private; `pub(crate)` chỉ khi cross-module; **không `pub`**.
- `xtask check-visibility` fail nếu có `pub` ngoài allowlist (`main.rs` entry, nếu cần).
- `pub` chỉ dành cho libs (`libs/*`) và API ổn định.
- `#[allow(dead_code)]` bị cấm; dùng `#[expect(dead_code, reason = "...")]` và xóa khi hết.
- `#[allow(unused_imports)]` bị cấm; sửa import.

### 8.3 Layer fitness functions

`xtask check-layering`:

- Input: `architecture.toml` chứa ma trận 6.3.
- Parse source bằng `syn` (hoặc regex có kiểm soát) để lấy `use crate::<layer>` và
  `crate::<layer>::`.
- Fail nếu cạnh không có trong ma trận.
- Cho phép `crate::model` mọi nơi (leaf).
- Allowlist tạm thời cho các vi phạm hiện có, kèm issue và ngày hết hạn; CI fail nếu
  allowlist quá hạn.

`xtask check-hot-path`:

- Scan `src/input`, `src/rendering`, `src/compositor` cho:
  - `services::config::load_`
  - `std::fs::`
  - `std::env::var`
  - `gtk4::`
  - `std::process::Command`
- Allowlist: `platform/`, `app/`, `shell/`; `rendering/blur/gl` cho GL; `platform/ffi` cho PAM.

`xtask check-unsafe`:

- Parse `unsafe` block; yêu cầu comment `// SAFETY:` ngay trước hoặc trong 3 dòng trước.
- Fail nếu thiếu.
- Báo cáo số unsafe theo module; đặt ngưỡng giảm dần.

`xtask check-brand`:

- Grep các biến thể `BabyDra`, `babydrade`, `babydra` trong code/docs; chỉ cho phép trong
  `platform::brand`, migration, và test.
- `typos` bắt lỗi chính tả; `rg` tùy chỉnh bắt naming.

### 8.4 CI pipeline

```yaml
# .github/workflows/ci.yml (phác thảo)
jobs:
  check:
    steps:
      - cargo fmt --all --check
      - cargo clippy --workspace --all-targets -- -D warnings
      - cargo test --workspace
      - cargo nextest run --workspace
      - cargo deny check
      - typos
      - cargo xtask check-layering
      - cargo xtask check-visibility
      - cargo xtask check-hot-path
      - cargo xtask check-unsafe
      - cargo llvm-cov --workspace --fail-under-lines 50
```

Yêu cầu:

- Pin toolchain; cache cargo/registry.
- CI < 10 phút; nếu lâu, tách job.
- Không cho phép `continue-on-error` cho các bước trên.
- Branch protection: require CI pass + 1 review.

### 8.5 Test policy

- **Unit test** cho logic thuần: geometry, layout, config parsing, state transitions.
- **Integration test** cho compositor state: map/unmap, focus, raise, maximize, minimize,
  fullscreen, drag/resize.
- **Golden test** cho titlebar/dock/frame element order.
- **Smoke test** cho backend/DRM/GTK (có thể manual trong CI runner đặc biệt).
- Không test phụ thuộc thời gian thực; dùng clock injection.
- Không `sleep` trong test; dùng deterministic clock.
- Test names mô tả hành vi: `drag_below_threshold_does_not_move_window`.

### 8.6 Unsafe policy

- Mọi `unsafe` block có `// SAFETY:` mô tả invariant, con trỏ, lifetime, thread-safety.
- Unsafe chỉ trong các module được allowlist:
  - `platform::ffi` (PAM)
  - `rendering::blur::gl` (GL)
  - `platform::drm` (DRM/GBM)
  - `platform::egl` (EGL surface)
- Safe wrapper API; không expose raw pointer ra ngoài module.
- `unsafe impl Send/Sync` phải có SAFETY + giải thích tại sao thread-safe.
- Chạy `cargo geiger`; review mọi unsafe mới.

### 8.7 Error policy

- Không `unwrap()`/`expect()` ngoài test, trừ invariant có ghi chú:
  ```rust
  let pointer = state.seat.get_pointer().expect("seat always has pointer");
  ```
  và được `#[expect(clippy::expect_used, reason = "...")]`.
- `let _ =` chỉ cho cleanup không quan trọng; còn lại `if let Err(err) = ... { warn!(...) }`.
- Lỗi ở biên dùng `thiserror`; `app` dùng `anyhow`/`Box<dyn Error>`.
- Không `catch_unwind` để che lỗi; nếu cần isolation, tách process.

### 8.8 Config policy

- Một `ConfigService`; không gọi `services::config::load_*` ngoài `platform`.
- Không đọc file/env trong `input`/`rendering`/`compositor`.
- `RuntimeConfig` bất biến; swap bằng `ArcSwap` ở frame boundary.
- Env overrides parse một lần; có test.
- Hot reload có debounce; không reload giữa frame.
- Config lỗi: log warn, giữ snapshot cũ; không panic.

### 8.9 Documentation policy

- Mỗi module có `//!`:
  - Trách nhiệm.
  - Layer được phép import.
  - Invariant.
- `docs/architecture.md` chứa:
  - Cây module.
  - Ma trận phụ thuộc.
  - State ownership.
  - Process model.
  - Link tới ADR.
- Mỗi quyết định lớn có ADR (`docs/adr/0001-layering.md`, ...).
- README phản ánh đúng tính năng và cách chạy.
- Public API (nếu có) có doc + example.

### 8.10 Review checklist

PR reviewer phải kiểm tra:

- [ ] Không thêm cạnh layer sai (CI sẽ bắt).
- [ ] Không thêm `pub` trong binary (CI sẽ bắt).
- [ ] Không thêm `unsafe` thiếu SAFETY (CI sẽ bắt).
- [ ] Không thêm `unwrap`/`expect`/`let _ =` mới ngoài allowlist.
- [ ] Không đọc config/file/env trong hot path.
- [ ] Có test cho logic mới.
- [ ] Docs/module docs được cập nhật.
- [ ] Không tăng số field của `BabydraState`/`AppState` (nếu có, justify).

---

## 9. Rủi ro và biện pháp giảm thiểu

| Rủi ro | Mức độ | Biện pháp |
|---|---|---|
| Refactor làm hỏng render/input | Cao | Characterization test trước; thay đổi nhỏ, compile/test sau mỗi bước; smoke matrix |
| Tách process GTK quá lớn | Cao | Làm sau giai đoạn 2; feature flag; giữ fallback in-process |
| Borrow checker khi tách state | Trung bình | Context hẹp; tránh `RefCell`; dùng `take()` có kiểm soát |
| API churn | Trung bình | Giữ alias/deprecation một chu kỳ; cập nhật docs |
| CI quá chậm | Trung bình | Cache; tách job; chỉ chạy layering/unsafe trên diff |
| Thiếu test GPU/DRM | Cao | Manual smoke; dùng Winit cho phần lớn test; CI runner đặc biệt nếu có |
| Naming migration ảnh hưởng user | Trung bình | Migration config; alias env; thông báo |
| `blur_all` fix thay đổi hành vi | Thấp | Test; mặc định false; document |

---

## 10. Tiêu chí nghiệm thu

### 10.1 Kiến trúc

- [ ] `compositor` không import `rendering`, `shell`, `gtk4`, `ui-kit`.
- [ ] `rendering` không import `compositor`, `shell`, `input`, `backends`.
- [ ] `input` không import `rendering`, `shell`, `backends`.
- [ ] `shell` không import `backends`; chỉ dùng port/trait của `compositor`.
- [ ] `compositor` không còn field `launcher`, `alt_tab_switcher`, `blur_engine`.
- [ ] `BabydraState`/`AppState` < 150 LOC; không có feature state.
- [ ] `xtask check-layering` pass 100%.

### 10.2 Chất lượng code

- [ ] `cargo fmt --check` pass.
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` pass.
- [ ] 0 `unsafe` thiếu `// SAFETY`.
- [ ] 0 `unwrap/expect` ngoài test/allowlist.
- [ ] 0 `let _ =` cho lỗi có thể xử lý.
- [ ] 0 `pub` trong binary ngoài allowlist.
- [ ] 0 dead code/alias stale.

### 10.3 Hành vi

- [ ] `blur_all` hoạt động; có test.
- [ ] Screenshot hoạt động đúng trên multi-output Udev.
- [ ] Titlebar render và hit-test dùng cùng geometry; có test.
- [ ] Frame element order giống nhau giữa Winit/Udev; có test.
- [ ] Config reload không đọc file trong hot path.
- [ ] Metadata không trích xuất lại trong render/blur/switcher.
- [ ] Lock/screenshot/clipboard không chặn frame loop.

### 10.4 Vận hành

- [ ] CI chạy < 10 phút; branch protection bật.
- [ ] Coverage đạt ngưỡng tối thiểu.
- [ ] Smoke matrix pass trên Winit + Udev.
- [ ] README/docs khớp code.
- [ ] `cargo deny`/`typos` pass.

---

## 11. Phụ lục

### A. File > 300 LOC

| LOC | File |
|---:|---|
| 606 | `rendering/blur/gl/pipeline.rs` |
| 514 | `extensions/lock/render.rs` |
| 496 | `compositor/input/touch/events.rs` |
| 431 | `compositor/state/switcher.rs` |
| 427 | `extensions/switcher/element.rs` |
| 420 | `rendering/blur/engine.rs` |
| 393 | `compositor/state/window/animation.rs` |
| 390 | `compositor/state.rs` |
| 346 | `extensions/launcher/state.rs` |
| 343 | `rendering/blur/gl/shaders.rs` |
| 335 | `compositor/state/window/sizing.rs` |
| 333 | `compositor/input/touchpad/swipe.rs` |
| 313 | `compositor/input/touch/gestures.rs` |
| 310 | `backends/udev/gpu.rs` |
| 310 | `compositor/input/pointer/button.rs` |
| 301 | `extensions/screenshot/editor/canvas/input.rs` |
| 301 | `extensions/screenshot/editor/handle/drawing.rs` |

### B. Bản đồ trùng lặp chi tiết

| # | Chủ đề | File A | File B | Cách xử lý |
|---|---|---|---|---|
| 1 | Frame composition | `backends/winit.rs:93-230` | `backends/udev/render.rs:122-246` | `rendering::frame::FrameComposer` |
| 2 | Drag/resize | `compositor/input/pointer/drag.rs` | `compositor/input/touch/drag.rs` | `input::drag::DragSession` + policy |
| 3 | Button/touch activation | `compositor/input/pointer/button.rs` | `compositor/input/touch/events.rs` | `input::activation` + `TitlebarActionDispatcher` |
| 4 | `render_pixels` | `extensions/launcher/buffer.rs` | `extensions/switcher/element.rs` | `rendering::buffer` |
| 5 | Backdrop gradient | `extensions/launcher/buffer.rs` | `extensions/switcher/element.rs` | `rendering::backdrop` / `ui_kit` |
| 6 | `init_layer_window` | `extensions/lock/render.rs:15` | `extensions/screenshot/editor/layer_shell.rs:7` | `platform::gtk::init_layer_window` |
| 7 | Titlebar geometry | `rendering/titlebar/mod.rs` | `rendering/titlebar/hit_test.rs` | `model::TitlebarLayout` |
| 8 | app_id/title | `compositor/state/active_window.rs:11` | `compositor/state/switcher.rs:302`, `rendering/blur/engine.rs:170` | `WindowMetadataCache` |
| 9 | Animation frame | `compositor/state/window/animation.rs` | `rendering/elements/window.rs` | `model::WindowVisualState` |
| 10 | EGL restore | `backends/winit.rs:187-202` | — | `FrameComposer`/`CaptureService` |

### C. Đề xuất thứ tự commit/PR

1. `chore: add rust-toolchain, workspace lints, CI, xtask skeleton`
2. `refactor: extract model::TitlebarLayout; use in render + hit-test`
3. `fix: honor blur_all; add tests`
4. `refactor: extract FrameComposer; dedupe winit/udev frame loops`
5. `fix: consume screenshot request once per frame in udev`
6. `refactor: extract shared Cairo buffer helpers`
7. `refactor: add WindowMetadataCache; remove repeated with_states`
8. `refactor: add ConfigService + ArcSwap; remove hot-path load_gesture_config`
9. `refactor: introduce Overlay trait; move launcher/switcher state to shell`
10. `refactor: split InputState/WindowStore/RenderState; narrow input contexts`
11. `refactor: move modules to app/input/shell/platform`
12. `feat: separate babydra-shell process (feature-flagged)`
13. `chore: remove dead code; update docs/README`
14. `test: integration + golden tests; coverage gate`

Mỗi PR phải nhỏ, compile/test độc lập, không trộn refactor với thay đổi hành vi.

### D. Smoke test matrix

| Kịch bản | Winit | Udev/DRM | Ghi chú |
|---|---|---|---|
| Khởi động compositor | ✅ | ✅ | socket, env, autostart |
| Mở/đóng cửa sổ | ✅ | ✅ | map/unmap, cleanup |
| Focus/raise | ✅ | ✅ | click, keyboard |
| Drag/resize | ✅ | ✅ | pointer + touch |
| Maximize/minimize/fullscreen | ✅ | ✅ | animation |
| Launcher | ✅ | ✅ | open, click, keyboard |
| Switcher | ✅ | ✅ | Alt+Tab, click, scroll |
| Blur | ✅ | ✅ | `blur_all`, app rules |
| Screenshot | ✅ | ✅ | multi-output |
| Lock screen | ✅ | ✅ | PAM success/fail |
| Clipboard history | ✅ | ✅ | source app closes |
| Touch gestures | — | ✅ | 2/3 finger |
| Touchpad gestures | ✅ | ✅ | 3/4 finger |
| VT switch | — | ✅ | pause/resume |
| Output hotplug | — | ✅ | mode/scale |

### E. Glossary

| Thuật ngữ | Nghĩa |
|---|---|
| **Layer** | Tầng kiến trúc (`app`, `compositor`, `input`, ...) |
| **Port** | Trait/interface do layer thấp định nghĩa, layer cao implement |
| **Fitness function** | Kiểm tra tự động để đảm bảo thuộc tính kiến trúc |
| **FrameComposer** | Thành phần dựng danh sách render element cho một frame |
| **Overlay** | UI do compositor vẽ đè lên cửa sổ (launcher, switcher, lock, ...) |
| **RuntimeConfig** | Snapshot cấu hình bất biến dùng trong một frame/event |
| **WindowMetadata** | Dữ liệu app_id/title/app_name/pid/icon của cửa sổ |
| **TitlebarLayout** | Geometry + hit-test của titlebar |
| **God object** | Struct biết/quản lý quá nhiều trách nhiệm |

---

## 12. Kết luận

Crate `window-manager` có nền tảng tính năng tốt và đã chứng minh chạy được ở cả hai backend,
nhưng kiến trúc hiện tại đang bị **xói mòn bởi phụ thuộc vòng, god state và trùng lặp**.
Các vấn đề này không thể giải quyết bền vững chỉ bằng tài liệu hay quy ước review; cần
**hàng rào tự động** (CI + xtask + lints) và **lộ trình refactor tăng dần**.

Ưu tiên thực hiện:

1. **Ngay lập tức:** guardrails + sửa `blur_all` + screenshot flag + SAFETY comments.
2. **Ngắn hạn:** `FrameComposer`, `TitlebarLayout`, `WindowMetadataCache`, `ConfigService`.
3. **Trung hạn:** `Overlay` registry, tách state, phá chu trình.
4. **Dài hạn:** tách `shell` process, dọn naming/docs, coverage gate.

Nếu chỉ chọn một thay đổi kiến trúc để bắt đầu, hãy chọn **`FrameComposer` + ma trận phụ
thuộc được enforce bằng `xtask`**: nó vừa giảm trùng lặp lớn nhất, vừa tạo khuôn mẫu cho
các layer còn lại.
