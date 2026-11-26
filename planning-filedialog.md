# File Dialog — Planning chi tiết

> **Mục tiêu:** Sửa file dialog để hoạt động hoàn chỉnh, thay thế hoàn toàn `xdg-open`.

---

## 1. Kiến trúc hiện tại

### Flow hoạt động

```text
User gọi xdg-open <target>
  → symlink ~/.local/bin/xdg-open → BabyDra
  → main() → app::run() → commands::dispatch()
  → filedialog::dispatch(&args)
    → args[0] filename == "xdg-open"?
      → Có target: open_target(target) [services/filedialog/open.rs]
      → Không target: show_chooser() [tạo GTK Application mới]
        → ui_kit::file_chooser::show() → dialog.rs (838 lines)
```

### Các file liên quan

| File | Lines | Vai trò |
|:--|:--|:--|
| `crates/window-manager/src/modules/shell/filedialog.rs` | 65 | Dispatch xdg-open, show chooser |
| `crates/window-manager/src/app/bootstrap/commands.rs` | 16 | Entry dispatch |
| `libs/common/src/filedialog/mod.rs` | 219 | Data contracts: FileDialogOptions, FileFilter, FileDialogResult, QuickAccessPlace |
| `libs/services/src/system/filedialog/mod.rs` | 9 | Re-exports |
| `libs/services/src/system/filedialog/open.rs` | 234 | xdg-open logic: URI, file, MIME, desktop entry |
| `libs/services/src/system/filedialog/places.rs` | 80 | Sidebar bookmarks |
| `libs/services/src/system/filedialog/filter.rs` | 96 | Glob/MIME matching |
| `libs/ui-kit/src/components/modals/file_chooser/mod.rs` | 5 | Re-export show() |
| `libs/ui-kit/src/components/modals/file_chooser/dialog.rs` | 838 | Full GTK4 dialog UI |
| `.old/babydra-filedialog/src/main.rs` | 557 | Old standalone binary (archived) |
| `libs/services/src/ui/i18n/locales/file_dialog/{en,vi}.json` | 70 | i18n strings |

### Callers khác (gọi xdg-open command)

- `crates/features/desktop/src/grid/render.rs` (line 45, 92, 94) — Desktop grid opens files via `Command::new("xdg-open")`
- `crates/features/topbar/src/center/features/recording/controller/actions/area.rs` (line 15) — Recording opens file via xdg-open

---

## 2. Vấn đề phát hiện

### P0 — Critical: File dialog không hiển thị được

**Nguyên nhân gốc:** `show_chooser()` trong `filedialog.rs` tạo một `gtk4::Application` mới hoàn toàn với `app.run_with_args(&[])`. Nhưng binary `BabyDra` đã là một compositor — nó **không có GTK Application** running (nó dùng Smithay, không phải GTK application loop). Khi gọi `Application::builder().build()` và `app.run_with_args(&[])`, nó cố tạo một GTK app process mới **trong cùng process** đang là compositor → xung đột.

Cụ thể:
1. **Không có GTK main loop** khi gọi `xdg-open` không có target — `app.run_with_args(&[])` chạy GTK main loop mới nhưng compositor đã có event loop riêng (Smithay).
2. **Window không có Wayland surface parent** — Dialog `Window::new()` trong `show_chooser()` tạo một GTK window top-level nhưng compositor không phải GTK app, không có parent window.
3. Khi gọi `xdg-open <file>`, hàm `open_target()` hoạt động tốt vì nó chỉ spawn process (không cần GTK).

### P1 — `show()` function tạo GTK `Window::new()` nhưng không có transient parent

- `dialog.rs` line 94: `let window = Window::new()` — tạo GTK Window mà không attach vào bất kỳ GTK Application nào.
- Khi gọi từ `show_chooser()`, nó nằm trong `app.connect_activate()` callback nên có Application context.
- Nhưng khi gọi từ external caller (ví dụ portal request), window không thuộc Application nào → có thể crash hoặc không hiển thị.

### P2 — `show_chooser()` gọi `std::process::exit(0)` trong callback

- Line 61: `std::process::exit(0)` — Nếu compositor process gọi xdg-open không target, nó sẽ **kill toàn bộ compositor** khi user chọn file.

### P3 — Không có XDG Desktop Portal implementation

- Old binary (`.old/babydra-filedialog`) có portal D-Bus service (`--portal` flag) nhưng code mới **không có**.
- Apps dùng `org.freedesktop.portal.FileChooser` D-Bus interface sẽ fallback về GTK FileChooserDialog hoặc fail.
- Điều này có nghĩa: Firefox, Electron apps, Flatpak apps **không thể mở file dialog** native.

### P4 — `show_chooser()` không forward kết quả

- Sau khi user chọn file, nó gọi `open_target()` và `std::process::exit(0)` — không output path ra stdout.
- Nếu caller expect path output (như tool pipeline), sẽ không nhận được gì.

### P5 — dialog.rs quá lớn (838 lines)

- Toàn bộ UI, state, navigation, rendering trong 1 file → vi phạm 200-line rule.

### P6 — Desktop grid/recording gọi `Command::new("xdg-open")` trực tiếp

- `desktop/src/grid/render.rs` gọi `xdg-open` command → spawn process mới.
- Nên gọi `services::filedialog::open::open_target()` trực tiếp thay vì spawn binary.

---

## 3. Plan sửa

### Phase 1: Sửa xdg-open dispatch (P0, P2, P4) — MUST

**File:** `crates/window-manager/src/modules/shell/filedialog.rs`

**Thay đổi:**
1. Tách logic `show_chooser()` thành subprocess thay vì inline GTK Application:
   - Khi không có target, **spawn chính binary** với flag `--filedialog-chooser` thay vì tạo GTK App inline.
   - Subprocess sẽ chạy GTK Application riêng, hiển thị dialog, output path ra stdout, rồi exit.
   - Compositor process chờ stdout, rồi gọi `open_target()` với kết quả.

2. Thêm dispatch cho `--filedialog-chooser` flag trong `commands.rs`:
   ```rust
   if args.contains("--filedialog-chooser") {
       filedialog::run_chooser_standalone(options);
       return true;
   }
   ```

3. Xóa `std::process::exit(0)` khỏi callback — thay bằng window.close() + quit application.

**File:** `crates/window-manager/src/app/bootstrap/commands.rs`

**Thay đổi:**
1. Thêm dispatch cho `--filedialog-chooser` và các CLI flags (`-f`, `-d`, `-m`, `-s`, `-t`, `-p`, `-n`, `--filter`).
2. Giữ `dispatch()` clean — gọi vào module function.

### Phase 2: Thêm CLI interface đầy đủ (tái sử dụng từ old binary)

**File:** `crates/window-manager/src/modules/shell/filedialog.rs`

**Thay đổi:**
1. Thêm function `run_cli(args: &[String])` parse CLI args (port từ `.old/babydra-filedialog`):
   - `-f/--open-file` — Open single file
   - `-d/--open-folder` — Open folder
   - `-m/--open-multiple` — Open multiple files
   - `-s/--save-file` — Save file
   - `-t/--title TITLE` — Custom title
   - `-p/--path PATH` — Initial folder
   - `-n/--name NAME` — Initial filename
   - `--filter SPEC` — Filter spec
   - `--open TARGET` — xdg-open mode

2. Output: in selected path(s) ra stdout, exit 0 (success) hoặc 1 (cancelled).

### Phase 3: Tách dialog.rs (P5) — SHOULD

**File:** `libs/ui-kit/src/components/modals/file_chooser/dialog.rs` (838 lines)

**Tách thành:**
```
file_chooser/
├── mod.rs          # Re-export show()
├── dialog.rs       # show() function + DialogState + confirm/cancel logic (~200 lines)
├── nav.rs          # navigate_to, breadcrumbs, back/forward/up (~120 lines)
├── sidebar.rs      # Places sidebar build (~60 lines)
├── list.rs         # File list rendering: build_file_row, load_rows, visible_rows (~150 lines)
├── toolbar.rs      # Header, nav row, bottom bar build (~120 lines)
└── format.rs       # file_icon, format_size, format_time, breadcrumb_segments (~80 lines)
```

### Phase 4: Inline open_target calls (P6) — NICE TO HAVE

**Files:**
- `crates/features/desktop/src/grid/render.rs` — Replace `Command::new("xdg-open").arg(&p).spawn()` with `services::filedialog::open::open_target(&p.to_string_lossy())`
- `crates/features/topbar/src/center/features/recording/controller/actions/area.rs` — Same replacement

**Lợi ích:** Không spawn thêm process, dùng trực tiếp logic trong libs.

### Phase 5: XDG Desktop Portal (P3) — FUTURE

**Phạm vi:** Port `--portal` mode từ `.old/babydra-filedialog` vào compositor hoặc tạo D-Bus service riêng.

**Chi tiết:**
1. Implement `org.freedesktop.portal.FileChooser` D-Bus interface.
2. Register D-Bus service khi compositor start.
3. Khi app gọi portal → show native file chooser → return result qua D-Bus.

**Lý do defer:** Cần `zbus`/`tokio` integration, phức tạp. Hiện tại GIO/GTK apps vẫn dùng được `open_target()`.

---

## 4. Thứ tự thực hiện

| # | Task | Files thay đổi | Độ phức tạp |
|:--|:--|:--|:--|
| 1 | Sửa `show_chooser()` spawn subprocess | `filedialog.rs`, `commands.rs` | Trung bình |
| 2 | Thêm CLI args parsing | `filedialog.rs` | Thấp |
| 3 | Fix `std::process::exit` trong dialog callback | `filedialog.rs` | Thấp |
| 4 | Tách `dialog.rs` thành modules | `file_chooser/*.rs` | Trung bình |
| 5 | Inline `open_target` calls | `grid/render.rs`, `area.rs` | Thấp |
| 6 | XDG Portal | Mới hoàn toàn | Cao (defer) |

---

## 5. Acceptance Criteria

- [ ] `xdg-open https://google.com` — mở browser ✓
- [ ] `xdg-open /home/user/Documents` — mở file manager ✓
- [ ] `xdg-open /home/user/file.txt` — mở notepad ✓
- [ ] `xdg-open /home/user/image.png` — mở preview ✓
- [ ] `xdg-open` (không target) — hiển thị file chooser dialog, chọn file → mở
- [ ] `BabyDra --filedialog-chooser` — hiển thị file chooser, output path stdout
- [ ] `BabyDra --filedialog-chooser -d` — folder picker
- [ ] `BabyDra --filedialog-chooser -s -n "untitled.txt"` — save dialog
- [ ] Desktop grid double-click file → mở đúng app
- [ ] Dialog đóng bình thường (không crash compositor)
- [ ] `cargo check --workspace` clean
- [ ] `cargo clippy -- -D warnings` clean
