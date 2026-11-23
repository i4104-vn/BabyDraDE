# Báo cáo đánh giá kiến trúc crate `window-manager`

- **Ngày đánh giá:** 23/11/2026
- **Mã nguồn khảo sát:** working tree tại commit nền `67514b5`, nhánh `main`
- **Phạm vi:** toàn bộ `crates/window-manager/src`, manifest của crate, các thư viện workspace mà crate phụ thuộc trực tiếp, `docs/architecture.md`, `docs/testing.md`, `docs/adr/0001-layering.md`, `docs/adr/0003-clean-architecture.md`
- **Tính chất:** đánh giá kiến trúc tĩnh; không thay đổi mã nguồn; báo cáo chỉ chứa mô tả và khuyến nghị

**Cách đọc nhanh:** nếu chỉ có 10 phút, đọc mục 0 (tóm tắt), 2.5 (chu trình), 4.1 (bảng phát hiện), 9 (kế hoạch di chuyển) và 10 (lộ trình).

### Mục lục

- 0. Tóm tắt cho người quyết định
- 1. Phạm vi, phương pháp và giới hạn
- 2. Số liệu hiện trạng
- 3. Đánh giá theo tầng
- 4. Phát hiện và xếp hạng
- 5. Vấn đề đặt tên và tổ chức thư mục
- 6. Vấn đề phân chia mã và trùng lặp
- 7. Rủi ro
- 8. Kiến trúc đích
- 9. Kế hoạch di chuyển và đổi tên chi tiết
- 10. Lộ trình thực hiện
- 11. Tiêu chí nghiệm thu
- 12. Kết luận

---

## 0. Tóm tắt cho người quyết định

`crates/window-manager` đã được đổi tên thư mục và chia module theo mô hình Clean Architecture, nhưng **ranh giới phụ thuộc thực tế vẫn chưa được thiết lập**. Đây là kiến trúc “Clean Architecture trên giấy”: tên tầng đúng, chiều phụ thuộc sai, và không có hàng rào tự động nào ngăn cản sự xói mòn tiếp theo.

Ba số liệu mô tả rõ nhất khoảng cách giữa ý định và hiện trạng:

- **85,4% mã nguồn nằm trong `adapters`**, trong khi `domain` và `application` cộng lại chỉ chiếm **6,4%**. Phần lõi quá mỏng để sở hữu các quyết định nghiệp vụ; phần lớn quyết định đang nằm trong adapter.
- **Ba chu trình phụ thuộc hai chiều** vẫn tồn tại giữa các adapter: `compositor` ↔ `rendering`, `compositor` ↔ `input`, `compositor` ↔ `shell`.
- **`BabydraState` là god object 39 trường**, với 10 khối `impl` nằm rải rác trong 10 tệp khác nhau, trộn protocol Wayland, danh sách cửa sổ, trạng thái input, overlay, clipboard, cấu hình và engine blur.

Kết luận ngắn: **chưa đạt Clean Architecture ở mức ranh giới phụ thuộc; đạt mức trung bình về tổ chức thư mục; có nền tảng tính năng và tài liệu tốt**. Không nên viết lại từ đầu, nhưng không nên tiếp tục thêm tính năng trên cấu trúc hiện tại trước khi xử lý các điểm P0.

### 0.1. Bảng điểm nhanh

| Tiêu chí | Điểm / 10 | Kết luận |
| --- | ---: | --- |
| Tuân thủ quy tắc phụ thuộc một chiều | 2 | Ba chu trình adapter; `input` còn phụ thuộc một chiều vào `shell` |
| Độ tinh khiết của `domain` | 3 | `domain` import Smithay và kiểu protocol; có cache toàn cục |
| Độ dày và vai trò của `application` | 2 | 409 dòng, chủ yếu là hàm thuần; không có use case điều phối |
| Cohesion của adapter | 3 | God state; `shell` trộn ba ứng dụng GTK với overlay compositor |
| Đặt tên và tổ chức thư mục | 5 | Có chia theo tính năng, nhưng nhiều tên mơ hồ và trùng nghĩa |
| Khả năng kiểm thử | 3 | Service locator, cache toàn cục, thiếu test cho luồng chính |
| An toàn lỗi thời gian chạy | 3 | 74 `unwrap`, 14 `expect`, 21 chỗ khóa mutex bằng `unwrap` |
| Khả năng thực thi kiến trúc | 1 | Không có CI, không có architecture test, không có coverage gate |
| **Tổng thể** | **3** | **Chưa đạt Clean Architecture vận hành; cần refactor có kiểm soát** |

### 0.2. Năm việc ưu tiên nhất

1. **Sửa lỗi `on_commit` bị gọi hai lần** trong cùng một commit của toplevel; đây là lỗi đúng đắn tiềm ẩn, không chỉ là vấn đề cấu trúc.
2. **Dựng hàng rào an toàn trước khi di chuyển mã**: CI, architecture test, và characterization test cho các luồng focus, sizing, input, commit.
3. **Phá ba chu trình** bằng cách tách `WindowRegistry`, `InputState`, `OverlayState`, `ClipboardState` khỏi `BabydraState`, và di chuyển metadata cache ra khỏi `domain`.
4. **Làm dày `application`**: đưa các quyết định focus/raise, sizing, switcher, placement, autostart từ adapter vào use case; thay port hàm tự do bằng trait có kết quả lỗi.
5. **Tách `adapters/shell` thành `adapters/desktop`**, tách ứng dụng GTK khỏi overlay compositor, và loại GTK khỏi state lõi.

---

## 1. Phạm vi, phương pháp và giới hạn

### 1.1. Phương pháp

- Đọc toàn bộ 123 tệp Rust nguồn và 26 tệp test trong `crates/window-manager/src`.
- Đối chiếu mã nguồn với mô hình Clean Architecture: quy tắc phụ thuộc một chiều, vai trò từng tầng, vị trí của quyết định nghiệp vụ, vị trí của hiệu ứng bên ngoài.
- Lập ma trận phụ thuộc bằng cách tìm các import giữa bốn khu vực adapter.
- Đo quy mô bằng số dòng, số tệp, số trường state, số khối `impl`, số test.
- Kiểm tra các mẫu rủi ro: `unwrap`, `expect`, bỏ qua kết quả bằng `let _`, khóa mutex, truy cập toplevel không kiểm tra, `#[allow(unused_imports)]`.
- Chạy `cargo check -p window-manager` và `cargo clippy -p window-manager --all-targets`: cả hai đều sạch. Vì vậy các phát hiện trong báo cáo là nợ kiến trúc, không phải lỗi biên dịch.

### 1.2. Giới hạn

- Không chạy kiểm thử giao diện, không chạy compositor trên DRM/Winit, không đo hiệu năng thực tế.
- Không đánh giá tính đúng đắn của thuật toán blur, PAM, screenshot hay định dạng ảnh; chỉ đánh giá vị trí và ranh giới của chúng.
- Số liệu dòng mã là số dòng vật lý, không phải độ phức tạp cyclomatic.
- Báo cáo không đề xuất thay đổi hành vi người dùng; mọi thay đổi cấu trúc phải giữ nguyên hành vi quan sát được.

---

## 2. Số liệu hiện trạng

### 2.1. Phân bố mã nguồn theo tầng

Không tính 26 tệp test:

| Tầng | Số tệp | Số dòng | Tỷ lệ | Nhận xét |
| --- | ---: | ---: | ---: | --- |
| `domain` | 6 | 574 | 3,7% | Quá nhỏ so với vai trò “entities + pure rules” |
| `application` | 7 | 409 | 2,7% | Mỏng hơn cả `domain`; thiếu use case điều phối |
| `adapters` | 93 | 13.152 | 85,4% | Chứa gần như toàn bộ quyết định và hiệu ứng |
| `infrastructure` | 12 | 1.008 | 6,5% | Backend, config, process |
| `composition` | 4 | 236 | 1,5% | Khởi tạo và wiring |
| `main.rs` | 1 | 17 | 0,1% | Rất mỏng, đúng vai trò entry point |
| **Tổng mã nguồn** | **123** | **15.396** | **100%** | |
| `src/tests` | 26 | 848 | — | Khoảng 5,5% so với mã nguồn |

Hệ quả: nếu một quyết định nghiệp vụ thay đổi, xác suất nó nằm trong adapter là khoảng 85%. Đây là dấu hiệu đảo ngược tỷ lệ mong muốn của Clean Architecture, nơi lõi nên chứa phần lớn luật và adapter chỉ dịch.

### 2.2. Phân bố bên trong `adapters`

| Khu vực | Số dòng | Tỷ lệ trong adapter | Vai trò hiện tại |
| --- | ---: | ---: | --- |
| `adapters/shell` | 3.980 | 30,3% | Launcher, switcher, lock, screenshot, clipboard picker |
| `adapters/compositor` | 3.226 | 24,5% | Protocol handler, state, window management, metadata |
| `adapters/input` | 3.224 | 24,5% | Keyboard, pointer, touch, touchpad, keymap |
| `adapters/rendering` | 2.711 | 20,6% | Render element, blur, titlebar, animation |
| `adapters/mod.rs` | 11 | 0,1% | Khai báo module |

`shell` là khu vực lớn nhất và cũng là khu vực trộn nhiều loại trách nhiệm nhất. Nó không chỉ là “shell” theo nghĩa Wayland; nó chứa cả ba ứng dụng GTK độc lập.

### 2.3. Các tệp lớn nhất

| Tệp | Số dòng | Ghi chú |
| --- | ---: | --- |
| `adapters/rendering/blur/gl/pipeline.rs` | 614 | Phức tạp kỹ thuật hợp lý, nhưng nên tách theo pass |
| `adapters/shell/lock/render.rs` | 491 | Trộn dựng giao diện, tài nguyên, trạng thái PAM |
| `adapters/input/touch/events.rs` | 477 | Trộn dispatch, drag/resize, focus, switcher, launcher |
| `adapters/compositor/state/switcher.rs` | 404 | Trộn model chọn cửa sổ, animation, GTK pixbuf |
| `adapters/rendering/blur/engine.rs` | 374 | Truy vấn metadata trực tiếp từ compositor |
| `adapters/compositor/state.rs` | 362 | God object và khởi tạo protocol |
| `adapters/shell/switcher/element.rs` | 350 | Render element của overlay |
| `adapters/rendering/blur/gl/shaders.rs` | 349 | Dữ liệu shader, ít rủi ro kiến trúc |
| `adapters/compositor/state/launcher.rs` | 349 | Trộn state launcher với buffer/render |
| `adapters/compositor/state/window/sizing.rs` | 333 | State machine maximize/fullscreen trong adapter |
| `adapters/compositor/state/window/animation.rs` | 320 | Trạng thái animation cửa sổ |
| `infrastructure/backends/udev/gpu.rs` | 312 | Khởi tạo GPU/DRM |
| `adapters/input/pointer/button.rs` | 311 | Dispatch chuột và lặp logic focus/raise |
| `adapters/shell/screenshot/editor/handle/drawing.rs` | 310 | Thao tác vẽ và lưu ảnh |
| `adapters/shell/screenshot/editor/canvas/input.rs` | 301 | Xử lý input canvas |

Có **21 tệp trên 250 dòng** và **4 tệp trên 400 dòng**. Không phải tệp lớn nào cũng xấu; nhưng các tệp lớn nằm ở `shell`, `input`, `state` thường là điểm giao thoa của nhiều trách nhiệm.

### 2.4. Chỉ số rủi ro

Các số dưới đây không tính tệp test, trừ khi ghi chú:

| Chỉ số | Số lượng | Ý nghĩa |
| --- | ---: | --- |
| `unwrap` | 74 | Có thể panic trong luồng sự kiện |
| `expect` | 14 | Chủ yếu ở khởi tạo; một số nằm trong vòng lặp sự kiện |
| Bỏ qua kết quả bằng `let _` | 38 | Bỏ qua lỗi có chủ đích hoặc vô tình |
| Khóa mutex bằng `unwrap` | 21 | Poisoned mutex làm sập compositor |
| Truy cập `toplevel` không kiểm tra | 3 | Race với cửa sổ đang đóng có thể panic |
| `#[allow(unused_imports)]` | 2 | Che giấu import chết trong `state/window/mod.rs` |
| Khối `impl BabydraState` | 10 | Trải trên 10 tệp, thuộc nhiều tầng |
| Trường của `BabydraState` | 39 | 38 `pub(crate)` và 1 trường private |
| Số hàm test | 50 | Trong 26 tệp, chủ yếu kiểm tra hàm thuần |
| Chu trình phụ thuộc adapter | 3 | Xem mục 2.5 |
| Fallback kích thước output `1920x1080` | 3 | Lặp trong pointer, touch, motion |
| Hai hệ thống shortcut | 2 | Hardcode và keymap cấu hình được |
| CI workflow | 0 | Tài liệu xác nhận không có CI |
| Phụ thuộc trực tiếp `wayland-sys` | 1 | Không có tham chiếu `wayland_sys` trong mã nguồn |

### 2.5. Ma trận phụ thuộc và các chu trình

Bảng dưới đây đếm số tham chiếu import giữa các khu vực adapter (không phải số tệp duy nhất):

| Từ \ Đến | `compositor` | `input` | `rendering` | `shell` |
| --- | ---: | ---: | ---: | ---: |
| `compositor` | — | 2 | 5 | 3 |
| `input` | 19 | — | 0 | 6 |
| `rendering` | 2 | 0 | — | 0 |
| `shell` | 3 | 0 | 2 | — |

Ba chu trình hai chiều:

1. **`compositor` ↔ `rendering`**: `compositor/state.rs` giữ `BlurEngine`; `compositor/state/launcher.rs` dùng `render_pixels` và `CustomRenderElements`; ngược lại `rendering/blur/engine.rs` gọi `compositor::state::metadata::get`.
2. **`compositor` ↔ `input`**: `compositor/state.rs` giữ `OverlayTouchState` của input; `compositor/state/switcher.rs` gọi `input::keyboard::focus`; ngược lại input dùng `BabydraState` ở 19 điểm.
3. **`compositor` ↔ `shell`**: state launcher/switcher dùng buffer, draw và kiểu `AltTabSwitcher` từ shell; shell switcher dùng lại `AltTabEntry` và `AltTabSwitcher` từ compositor.

Ngoài ra có một phụ thuộc một chiều đáng lưu ý: **`input` → `shell`** 6 điểm. Input không nên biết đến overlay desktop; đây là dấu hiệu thiếu một ranh giới trung gian (intent/command) giữa input và UI.

### 2.6. Đối chiếu tài liệu với mã nguồn

| Tuyên bố trong tài liệu | Thực tế trong mã nguồn | Kết luận |
| --- | --- | --- |
| `domain` là leaf, không protocol handle, không toolkit | `domain/cursor.rs` import `smithay::input::pointer::CursorIcon` và `xdg_toplevel::ResizeEdge`; `domain/window.rs` dùng Smithay geometry và cache toàn cục | Sai một phần |
| `application` framework-free | `application/windows.rs` dùng `smithay::utils`; `application/gestures.rs` dùng `common`; `application/shortcuts.rs` giữ `OnceLock` và `RwLock` | Sai một phần |
| Các chu trình cũ không còn trong thiết kế | Ba chu trình vẫn tồn tại trong import thực tế | Đúng về ý định, sai về hiện trạng |
| Port là “plain function types” | Đúng, nhưng đây là điểm yếu: không trait, không `Result`, không mock | Đúng nhưng chưa đủ |
| Ranh giới được duy trì bởi review và `pub(crate)` | Không có CI, không có architecture test; chu trình tồn tại chứng minh review chưa đủ | Chưa hiệu quả |
| Test nằm trong `src/tests`, mirror source, gắn bằng `#[path]` | Đúng; 26 tệp, 50 test | Đúng, nhưng mô hình test có vấn đề riêng |
| `composition` chỉ là entry point + wiring | `run()` còn spawn autostart, nối clipboard channel, dò môi trường host, khởi tạo GTK, quản lý vòng đời process con | Sai một phần |

Điểm quan trọng: ADR 0003 mô tả **ý định** của một cuộc refactor, không mô tả **trạng thái** của mã nguồn sau refactor. Tài liệu đang tạo cảm giác an toàn giả.

---

## 3. Đánh giá theo tầng

### 3.1. `domain`

**Điểm tốt**

- `domain/keysyms.rs`, `domain/shortcuts.rs`, `domain/titlebar.rs` chứa luật tương đối thuần và có test.
- `TitlebarLayout` và `WindowVisualState` là các value object dễ hiểu.
- Tài liệu module mô tả rõ mong muốn “không đọc config, không chạy process, không nói chuyện protocol”.

**Vấn đề**

- `domain/cursor.rs` ánh xạ trực tiếp `ResizeEdge` của Wayland sang `CursorIcon` của Smithay. Đây là kiến thức giao diện/protocol, không phải luật nghiệp vụ. Nó khiến domain không thể dùng trong môi trường không có Smithay và làm test phải kéo theo thư viện nặng.
- `domain/window.rs` chứa `OnceLock<RwLock<HashMap<...>>>` làm cache metadata toàn cục. Cache là trạng thái có vòng đời, không phải luật. Nó gây ô nhiễm test, che giấu dependency, và có thể giữ metadata cũ sau khi surface bị hủy nếu `forget` không được gọi đúng.
- `domain/window.rs` dùng trực tiếp `smithay::utils` cho `WindowVisualState`. Bản thân geometry có thể chấp nhận được như một value object, nhưng nếu mục tiêu là domain thuần thì nên có kiểu hình học nội bộ và adapter chuyển đổi.
- Cache bỏ qua lỗi khóa bằng `if let Ok(...)`: nếu mutex bị poison, dữ liệu cũ vẫn tồn tại và lỗi bị nuốt im lặng.
- Khóa cache là chuỗi được tạo từ `format!("{:?}", surface.id())` ở adapter. Đây là khóa opaque, khó kiểm tra tính duy nhất và dễ vỡ nếu cách Debug của id thay đổi.

**Khuyến nghị**

- Thay `cursor.rs` bằng một enum `CursorShape` thuần trong domain; việc ánh xạ sang `CursorIcon` và `ResizeEdge` đặt trong `adapters/wayland/cursor.rs`.
- Giữ `WindowMetadata` như một value object thuần; chuyển cache sang `adapters/window/metadata.rs` hoặc một `WindowRegistry` thuộc adapter.
- Nếu vẫn muốn cache trong domain, phải biến nó thành một thành phần được inject, có vòng đời rõ ràng, không dùng static; nhưng phương án tốt hơn là đưa ra khỏi domain.
- Thêm test khẳng định domain không import Smithay, GTK, `std::fs`, `std::process`.

### 3.2. `application`

**Điểm tốt**

- `gestures.rs` là ví dụ đúng hướng: nhận input thô, trả về `WindowIntent`, không phụ thuộc trạng thái compositor.
- `clipboard.rs` tách chính sách chọn MIME khỏi pipe/thread/Wayland selection.
- `windows.rs` đã tách một phần toán học maximize/restore khỏi protocol.
- Có test cho các hàm thuần.

**Vấn đề**

- Chỉ 409 dòng cho toàn bộ use case của một compositor là quá mỏng. Phần lớn quyết định vẫn nằm trong adapter: chọn cửa sổ active, thứ tự z-order, focus, maximize/fullscreen state machine, đặt cửa sổ mới, switcher ordering, autostart.
- Các “use case” hiện tại là hàm tự do, không phải đối tượng điều phối. Không có nơi nào sở hữu trình tự: kiểm tra điều kiện, gọi port, cập nhật state, phát intent.
- `ports.rs` dùng alias con trỏ hàm, ví dụ một hàm nhận chuỗi lệnh hoặc một hàm trả về danh sách shortcut. Cách này có ba hệ quả:
  - không thể trả lỗi có cấu trúc;
  - không thể mock theo hành vi, chỉ có thể thay bằng hàm khác;
  - không thể biểu diễn lifecycle, trạng thái, hoặc nhiều implementation.
- `GesturePolicy` là DTO cấu hình nằm trong `ports.rs`, trộn mô hình cấu hình với định nghĩa cổng.
- `ShortcutRegistry` dùng `OnceLock` toàn cục và gọi `get` kèm `expect`. Đây là service locator: application tự lấy dependency từ global state thay vì nhận qua constructor. Nó làm test phụ thuộc thứ tự khởi tạo và biến lỗi thiếu khởi tạo thành panic.
- `ShortcutRegistry` giữ một khóa đọc/ghi bọc danh sách shortcut; watcher thread và luồng xử lý sự kiện của compositor cùng đọc/ghi. Không có mô hình đồng bộ rõ ràng, không có xử lý poisoned lock.
- `application` vẫn import Smithay geometry và `common`, nên tuyên bố “framework-free” không đúng.

**Khuyến nghị**

- Tổ chức lại thành `application/use_cases/` với các đối tượng hoặc service có dependency inject:
  - `WindowActions` (close, minimize, maximize, fullscreen, restore);
  - `FocusService` (active window, cycle, raise);
  - `SwitcherService` (ordering, selection, commit/cancel);
  - `PlacementService` (vị trí cửa sổ mới, restore, output);
  - `ClipboardService` (chọn MIME, capture, lịch sử);
  - `ShortcutService` (nạp, reload, khớp phím, dispatch);
  - `AutostartService` (đọc danh sách, spawn, dừng).
- Thay port hàm bằng trait có `Result`, có fake implementation cho test.
- Đưa `GesturePolicy` sang `application/model/` hoặc `domain` nếu là luật thuần.
- Loại bỏ `OnceLock`; composition tạo service shortcut được chia sẻ và truyền vào input/runtime.
- Định nghĩa kiểu hình học nội bộ hoặc chấp nhận một module geometry thuần, không import Smithay.

### 3.3. `adapters`

**Điểm tốt**

- Protocol handler được tách theo protocol: `compositor`, `xdg_shell`, `layer_shell`, `xdg_decoration`.
- Rendering có phân tầng element/blur/animation/titlebar tương đối rõ.
- Input có chia keyboard/pointer/touch/touchpad.
- Một số logic thuần trong adapter có test.

**Vấn đề**

- `BabydraState` là trung tâm của mọi thứ: 39 trường, 10 khối `impl` trong 10 tệp. Nó vừa là protocol state, vừa là window registry, vừa là input state, vừa là overlay state, vừa là clipboard state, vừa là config cache, vừa là render resource.
- Các quyết định nghiệp vụ nằm rải rác:
  - `state/window/sizing.rs` quyết định maximize/fullscreen/restore;
  - `state/switcher.rs` quyết định thứ tự, selection, animation, đóng/mở;
  - `state/hit_test.rs` quyết định cửa sổ nào nhận click và action titlebar;
  - `handlers/xdg_shell.rs` quyết định vị trí cửa sổ mới và focus;
  - `input/pointer/button.rs` và `input/touch/events.rs` quyết định focus/raise/resize;
  - `composition/run()` quyết định autostart.
- `input` phụ thuộc `shell` để tính layout switcher và để trigger lock/screenshot/clipboard. Đây là phụ thuộc ngược chiều: thiết bị input không nên biết ứng dụng desktop.
- `shell` trộn hai loại thành phần có vòng đời khác nhau:
  - overlay do compositor vẽ (launcher, switcher);
  - ứng dụng GTK chạy trong cùng process (lock, screenshot editor, clipboard picker).
- `state/switcher.rs` giữ `gtk4::gdk_pixbuf::Pixbuf` và các `Tween`. State lõi của compositor đang phụ thuộc GTK và animation UI.
- `state/blur.rs` thực chất chỉ reload cấu hình blur; tên gợi ý rằng nó sở hữu engine blur.
- Config I/O nằm trong adapter: `input/keymap/config.rs` đọc/ghi file và spawn watcher thread; `pointer/config.rs` và `touchpad/config.rs` đọc cấu hình qua `services`.
- Hai hệ thống shortcut cùng tồn tại; hardcode chặn trước keymap cấu hình được.
- `handle_window_action` nhận chuỗi; giá trị không hợp lệ bị bỏ qua im lặng.
- `state/window/mod.rs` dùng `#[allow(unused_imports)]` để che import chết.

**Khuyến nghị**

- Tách state theo thành phần và để adapter chỉ dịch sự kiện, không ra quyết định nghiệp vụ.
- Di chuyển overlay UI và ứng dụng GTK sang `adapters/desktop`; lõi chỉ giữ dữ liệu tối thiểu (id cửa sổ, thứ tự, selection).
- Loại `Pixbuf` khỏi state lõi; nếu cần icon, chuyển thành texture/render resource do desktop overlay sở hữu.
- Đưa config I/O và watcher về `infrastructure/config`.
- Hợp nhất shortcut thành một dispatcher.
- Dùng enum hành động có kiểu, parse chuỗi tại biên IPC/CLI.

### 3.4. `infrastructure`

**Điểm tốt**

- Backend Winit và Udev/DRM được tách khỏi phần còn lại.
- `process.rs` thực hiện port launch command.
- `config.rs` chuyển đổi mô hình cấu hình sang policy của application.

**Vấn đề**

- `backends/winit.rs` và `backends/udev/render.rs` lặp lại cùng một vòng lặp render: lấy frame, xử lý screenshot request, dựng element, commit. Chênh lệch chủ yếu ở cách lấy surface/DRM.
- Backend biết quá nhiều về UI: gọi trực tiếp `shell::screenshot::capture::take_requested` và `capture_screen`. Đây nên là một port/use case, không phải chi tiết backend.
- `infrastructure` phụ thuộc `adapters` (`CalloopData`, `BabydraState`, `Cursor`). Theo thứ tự tầng đã tuyên bố thì điều này được phép, nhưng nó khiến backend không thể test độc lập và làm “infrastructure” không còn là lớp implement port thuần túy.
- `wayland-sys` là dependency trực tiếp nhưng không có tham chiếu trong mã nguồn.
- `infrastructure/config.rs` chỉ có một hàm chuyển đổi; phần lớn config thật vẫn nằm trong `services` và adapter. Tên `infrastructure/config` gợi ý rằng đây là nơi duy nhất đọc cấu hình, nhưng không đúng.

**Khuyến nghị**

- Tách `backends/render_loop.rs` dùng chung; backend chỉ cung cấp surface, renderer, output và sự kiện.
- Đưa screenshot request thành intent/port để backend không gọi trực tiếp shell.
- Gom toàn bộ đọc/ghi cấu hình, watcher, migration vào `infrastructure/config`.
- Xóa `wayland-sys` nếu xác nhận không dùng trực tiếp; kiểm tra lại qua `cargo udeps` hoặc tìm kiếm toàn workspace.

### 3.5. `composition`

**Điểm tốt**

- `main.rs` chỉ 17 dòng; `dispatch()` tách subcommand; `bootstrap` tách khởi tạo tracing/môi trường.
- Có ý thức về thứ tự khởi tạo: host display trước, GTK sau, export socket sau.

**Vấn đề**

- `run()` ôm quá nhiều trách nhiệm:
  - khởi tạo shortcut registry và watcher;
  - dò biến môi trường host;
  - tạo event loop, display, state;
  - khởi tạo GTK;
  - tạo clipboard channel và đăng ký source;
  - gọi backend;
  - export socket;
  - spawn autostart và kill process con khi thoát.
- Autostart là một use case (đọc danh sách, lọc comment, spawn, quản lý vòng đời), không phải wiring.
- Clipboard channel là chi tiết của adapter clipboard, không nên nằm trong `run()`.
- Lời gọi `expect` khi đăng ký clipboard completion channel biến lỗi cấu hình runtime thành panic.
- Không có struct `Runtime`/`CompositionRoot` sở hữu vòng đời; khó test, khó thay thế backend, khó thêm shutdown hook.

**Khuyến nghị**

- Tách `composition/runtime.rs` (vòng đời event loop, shutdown) và `composition/wiring.rs` (xây dependency).
- Đưa autostart sang `application/autostart` với port process.
- Đưa clipboard wiring vào builder của adapter clipboard.
- Dò môi trường host vào `composition/bootstrap/env.rs`.
- Trả `Result` có ngữ cảnh thay vì `expect` trong luồng khởi tạo.

### 3.6. Test, CI và tài liệu

**Điểm tốt**

- 50 test, 26 tệp; có test cho gesture, clipboard, titlebar, shortcut, geometry, blur projection, config.
- `nextest.toml` cấu hình chạy deterministic.
- `docs/testing.md` có mục tiêu coverage và smoke matrix rõ ràng.

**Vấn đề**

- Tỷ lệ test trên mã nguồn thấp: 848 dòng test so với 15.396 dòng nguồn.
- Mô hình `src/tests` + `#[path]` là không chuẩn:
  - test được biên dịch như một phần của module nguồn;
  - di chuyển tệp nguồn kéo theo sửa đường dẫn `#[path]`;
  - công cụ đo coverage và IDE có thể không nhận đúng;
  - test không thể kiểm tra hành vi qua biên module một cách tự nhiên.
- Test hiện có chủ yếu kiểm tra hàm thuần, không kiểm tra:
  - state machine sizing;
  - focus/raise;
  - dispatch chuột/touch;
  - `xdg_shell` handler;
  - `on_commit` hai lần;
  - config watcher;
  - composition/backend;
  - thứ tự ưu tiên shortcut.
- Không có CI. Tài liệu nói rõ “không có CI workflow”; coverage target trong `docs/testing.md` không được thực thi.
- Không có architecture test; đây là lý do trực tiếp khiến ba chu trình quay lại sau refactor.

**Khuyến nghị**

- Chuyển test module về `#[cfg(test)]` ngay trong tệp nguồn, hoặc dùng `tests/` cho integration test; bỏ `#[path]`.
- Thêm characterization test trước mỗi bước di chuyển.
- Dựng CI tối thiểu: fmt, clippy `-D warnings`, test, deny, architecture test.
- Thêm coverage gate theo tầng, bắt đầu ở mức thấp nhưng phải tăng dần.

---

## 4. Phát hiện và xếp hạng

### 4.1. Bảng tổng hợp

Thang mức độ:

- **P0 — Nghiêm trọng:** phải xử lý trước khi thêm tính năng; ảnh hưởng trực tiếp đến tính đúng đắn hoặc làm kiến trúc không thể tiến hóa.
- **P1 — Cao:** xử lý trong đợt refactor kế tiếp; gây trùng lặp, khó test, khó mở rộng.
- **P2 — Trung bình:** dọn dẹp có kế hoạch; ảnh hưởng khả năng đọc hiểu và bảo trì.
- **P3 — Thấp:** tinh chỉnh tên gọi, tài liệu, định dạng.

| ID | Mức | Phát hiện | Bằng chứng chính | Tác động |
| --- | --- | --- | --- | --- |
| P0-01 | P0 | Ba chu trình phụ thuộc giữa adapter | Ma trận 2.5; `compositor` ↔ `rendering`, `compositor` ↔ `input`, `compositor` ↔ `shell` | Không thể test/compile tách tầng; thay đổi lan rộng; không thể tách crate |
| P0-02 | P0 | `BabydraState` là god object | 39 trường, 10 khối `impl`, 362 dòng cho khai báo/khởi tạo/listener | Borrow-checker khó, merge conflict, che giấu phụ thuộc |
| P0-03 | P0 | `application` mỏng, quyết định nằm trong adapter | Application 409 dòng; sizing/switcher/focus/placement/autostart nằm trong adapter/composition | Luật nghiệp vụ không test được, bị khóa vào Smithay/GTK |
| P0-04 | P0 | `domain` và `application` không thực sự thuần | `domain/cursor.rs`, `domain/window.rs`, `application/windows.rs`, `OnceLock` trong application | Tuyên bố Clean Architecture sai; test phải kéo framework |
| P0-05 | P0 | `window.on_commit()` bị gọi hai lần trong một commit | `handlers/compositor.rs` và `handlers/xdg_shell.rs` cùng gọi cho toplevel | Xử lý state lặp, configure thừa, rủi ro animation/initial configure |
| P1-01 | P1 | Port là con trỏ hàm, không phải trait | `application/ports.rs` dùng alias hàm nhận chuỗi lệnh và alias hàm trả danh sách shortcut | Không trả lỗi, không mock hành vi, không có lifecycle |
| P1-02 | P1 | Service locator toàn cục trong application | `ShortcutRegistry` dùng `OnceLock` và `get` kèm `expect` | Test phụ thuộc thứ tự; panic nếu quên khởi tạo; hidden dependency |
| P1-03 | P1 | `shell` quá tải, trộn GTK app với overlay | 3.980 dòng; lock/screenshot/clipboard là app GTK; launcher/switcher là overlay | Không thể tách process/crate; GTK thấm vào lõi |
| P1-04 | P1 | Trùng lặp focus/raise, drag/resize, hit-test | `button.rs`, `touch/events.rs`, `focus.rs`, `switcher.rs`; pointer/touch drag | Sửa một chỗ dễ bỏ sót chỗ khác; hành vi lệch |
| P1-05 | P1 | Vòng lặp render bị nhân đôi | `backends/winit.rs` và `udev/render.rs` cùng flow | Backend mới phải copy thêm lần nữa; screenshot logic phân tán |
| P1-06 | P1 | Config I/O và watcher nằm trong adapter | `keymap/config.rs` đọc file, spawn thread; pointer/touchpad config đọc qua `services` | Không kiểm soát vòng đời; reload race; khó test |
| P1-07 | P1 | Xử lý lỗi bằng panic | 74 `unwrap`, 14 `expect`, 21 chỗ khóa mutex bằng `unwrap` | Một client xấu hoặc panic thread làm sập compositor |
| P1-08 | P1 | Hai hệ thống shortcut chồng nhau | `keyboard/shortcuts.rs` hardcode chặn trước `keymap` cấu hình | Người dùng không remap được; hai nguồn luật |
| P1-09 | P1 | Hành động cửa sổ dạng chuỗi, bỏ qua im lặng | `handle_window_action` nhận chuỗi; nhánh mặc định bỏ qua | Lỗi chính tả không báo; khó refactor; không exhaustive |
| P1-10 | P1 | Không có CI và architecture gate | 0 workflow; ADR 0003 xác nhận chỉ dựa vào review | Chu trình quay lại; coverage target không được thực thi |
| P1-11 | P1 | Test không phủ luồng chính | 50 test chủ yếu hàm thuần; thiếu sizing, focus, dispatch, handler, backend | Refactor không có lưới an toàn |
| P2-01 | P2 | Tên `shell`, `state`, `window`, `common`, `data`, `handle` mơ hồ | Nhiều module trùng nghĩa; xem mục 5 | Người mới dễ đặt sai chỗ; import vòng |
| P2-02 | P2 | `titlebar` xuất hiện ở ba tầng | `domain/titlebar.rs`, `state/titlebar.rs`, `rendering/titlebar/` | Không rõ luật nào thuộc tầng nào |
| P2-03 | P2 | Ngôn ngữ comment không thống nhất | Tiếng Việt trong `state/switcher.rs`, `shell/switcher/draw.rs`, `element.rs` | Khó đọc thống nhất; style review thiếu nhất quán |
| P2-04 | P2 | Dependency hygiene kém | `wayland-sys` không dùng; `animation` kéo GTK4; `common` trộn enum và design token | Build nặng, khó tái sử dụng, supply chain rộng |
| P2-05 | P2 | Tài liệu lệch hiện trạng | ADR 0003 và `docs/architecture.md` tuyên bố sạch hơn thực tế | Niềm tin sai vào kiến trúc; onboarding lệch |
| P2-06 | P2 | 21 tệp trên 250 dòng, 4 tệp trên 400 dòng | Xem 2.3 | Chi phí đọc hiểu và review tăng |
| P2-07 | P2 | Fallback output `1920x1080` lặp ba lần | `pointer/motion.rs`, `pointer/button.rs`, `touch/events.rs` | Hành vi không nhất quán khi không có output |
| P3-01 | P3 | `#[allow(unused_imports)]` và import chết | `state/window/mod.rs` | Che giấu dead code, làm mờ ranh giới |
| P3-02 | P3 | Tên tệp kỹ thuật chưa chính xác | `elements/types.rs`, `cursor/data.rs`, `state/window/state_data.rs` | Khó đoán nội dung từ tên |

### 4.2. Phân tích các phát hiện P0

#### P0-01. Ba chu trình phụ thuộc adapter

Đây là phát hiện nghiêm trọng nhất vì nó vô hiệu hóa toàn bộ lợi ích của Clean Architecture. Khi `compositor` import `rendering`, `rendering` import ngược `compositor` để lấy metadata, thì hai khu vực này thực chất là một khối. Khi `input` import `shell` để tính layout switcher, input không còn là bộ chuyển đổi thiết bị thuần túy; nó trở thành một phần của UI desktop.

Tác động cụ thể:

- Không thể biên dịch hoặc test `rendering` mà không kéo `compositor`.
- Không thể thay metadata provider bằng dữ liệu giả khi test blur; phải dựng cả `BabydraState`.
- Không thể tách `shell` thành crate/process riêng vì shell và compositor dùng chung kiểu `AltTabEntry`, `AltTabSwitcher`.
- Mọi thay đổi ở state compositor có thể buộc sửa input, rendering, shell; review không thể khoanh vùng.

Hướng phá vòng:

- `rendering` chỉ nhận metadata/visual state đã được chuẩn bị; không gọi ngược `compositor::metadata`.
- `input` chỉ nhận `WindowRegistry` và các intent/command; không import `shell`.
- `compositor`/runtime chỉ tổng hợp các thành phần; overlay state thuộc `desktop`, không nằm trong state lõi.
- Thêm architecture test chặn import giữa các cặp module.

#### P0-02. `BabydraState` là god object

39 trường của `BabydraState` có thể chia thành sáu nhóm trách nhiệm:

- protocol/runtime: display, socket, loop signal, các global Smithay, seat, popup;
- window registry: `space`, `windows`, active window, metadata;
- input: move/resize, pointer, touch, gesture buffer, last titlebar click;
- overlay: switcher, launcher;
- clipboard: pending, capture, sender;
- render/config: blur engine, natural scroll, gesture policy, loader.

Mười khối `impl BabydraState` nằm ở mười tệp thuộc các khu vực khác nhau. Điều này có nghĩa là “quyền truy cập state” không được kiểm soát: bất kỳ module nào cũng có thể thêm phương thức vào god object và chạm vào bất kỳ trường nào. Đó là cơ chế sinh ra chu trình.

Tác động:

- Borrow checker buộc phải tách nhỏ hàm hoặc dùng clone; code trở nên quanh co.
- Test một hành vi nhỏ phải dựng gần như toàn bộ compositor.
- Merge conflict tăng theo số người chạm vào state.
- Không thể nhìn vào một tệp để hiểu trọn một thành phần.

Hướng sửa: xem bảng phân rã state ở mục 6.1. Mục tiêu là `BabydraState` chỉ còn vai trò facade 6–7 trường, mỗi trường là một thành phần có API hẹp.

#### P0-03. `application` mỏng, quyết định nằm trong adapter

Clean Architecture không yêu cầu application phải nhiều dòng, nhưng nó yêu cầu application sở hữu **quyết định**. Hiện tại:

- Quyết định cửa sổ nào được focus sau click nằm trong `input/pointer/button.rs` và `input/touch/events.rs`.
- Quyết định thứ tự cửa sổ trong Alt+Tab nằm trong `state/switcher.rs`.
- Quyết định vị trí cửa sổ mới nằm trong `handlers/xdg_shell.rs`.
- Quyết định maximize/fullscreen/restore nằm trong `state/window/sizing.rs`.
- Quyết định autostart nằm trong `composition/run()`.
- Quyết định cửa sổ nào “active” bị lặp ở ba nơi với ba cách chọn khác nhau.

Vì quyết định nằm trong adapter, nó bị khóa vào Smithay, GTK và event loop. Muốn test phải dựng protocol state; muốn thay đổi chính sách phải sửa nhiều adapter; muốn tái sử dụng cho backend khác phải copy.

Hướng sửa: định nghĩa command/intent ở application, adapter chỉ thực thi. Ví dụ: application quyết định “focus cửa sổ X”, adapter gọi seat và gửi configure; application quyết định “cửa sổ mới ở vị trí Y”, adapter map element.

#### P0-04. `domain` và `application` không thực sự thuần

Đây là phát hiện có bằng chứng rõ ràng nhất về khoảng cách giữa tài liệu và mã nguồn:

- `domain/cursor.rs` phụ thuộc Smithay và Wayland protocol.
- `domain/window.rs` có cache toàn cục, trạng thái ẩn và vòng đời không rõ.
- `application/windows.rs` dùng `smithay::utils`.
- `application/gestures.rs` dùng `common::SwipeDirection`.
- `application/shortcuts.rs` dùng `OnceLock` và `RwLock`.

Hệ quả: không thể chạy test lõi trong môi trường không có Smithay; không thể tái sử dụng luật cho công cụ khác; không thể đảm bảo domain không bị ảnh hưởng bởi thay đổi protocol.

Hướng sửa: định nghĩa kiểu giá trị thuần trong domain/application; adapter chuyển đổi. Chấp nhận một module geometry thuần dùng chung, nhưng không import Smithay. Loại bỏ static state; inject cache/service.

#### P0-05. `on_commit` bị gọi hai lần

Luồng hiện tại:

- `CompositorHandler::commit` gọi `on_commit_buffer_handler`, tìm root toplevel và gọi `window.on_commit()`.
- Sau đó nó gọi `xdg_shell::handle_commit`.
- `handle_commit` lại tìm cửa sổ có toplevel surface trùng với surface commit và gọi `window.on_commit()` lần nữa.

Với một commit của root toplevel, cả hai nhánh đều đúng, nên `on_commit` chạy hai lần. Hậu quả có thể gồm:

- xử lý committed state lặp;
- gửi configure thừa;
- nhiễu animation/initial configure;
- sai lệch giữa trạng thái buffer và trạng thái hình học.

Ngay cả khi `on_commit` hiện tại idempotent, việc gọi hai lần là bất biến ngầm không được ghi lại và sẽ dễ vỡ khi nâng Smithay.

Hướng sửa: chọn một nơi duy nhất sở hữu `on_commit` cho toplevel. `handle_commit` chỉ nên xử lý popup, initial configure hoặc các việc không trùng. Thêm test hồi quy ở mức handler nếu có thể, hoặc ít nhất thêm comment bất biến và kiểm tra bằng review.

### 4.3. Phân tích các phát hiện P1

#### P1-01 và P1-02. Port yếu và service locator

Hai vấn đề này đi cùng nhau. Port dạng con trỏ hàm không thể biểu diễn lỗi, trạng thái hay lifecycle, nên application phải tự tìm dependency qua global. `ShortcutRegistry` là ví dụ điển hình: nó vừa là service, vừa là registry toàn cục, vừa là nơi giữ `RwLock`. Điều này làm test khó cô lập và che giấu thứ tự khởi tạo.

Hướng sửa: định nghĩa trait cho các cổng thực sự có hiệu ứng; composition tạo implementation và truyền vào service. Chỉ giữ hàm thuần cho logic không có hiệu ứng.

#### P1-03. `shell` quá tải

`adapters/shell` hiện chứa:

- overlay do compositor vẽ: launcher, switcher;
- ứng dụng GTK trong cùng process: lock screen, screenshot editor, clipboard picker;
- helper GTK dùng chung.

Ba loại này có vòng đời, mô hình lỗi và cách test khác nhau. Việc gộp chúng làm tên `shell` mất nghĩa và khiến GTK trở thành dependency của state lõi. Đây là rào cản lớn nhất để tách process/crate sau này.

#### P1-04. Trùng lặp focus/raise và drag/resize

Cùng một khối logic “raise window, set keyboard focus, set activated, gửi configure, update active window” xuất hiện trong:

- `input/pointer/button.rs` (hai lần: border resize và window click);
- `input/keyboard/focus.rs`;
- `input/touch/events.rs`;
- `state/switcher.rs`.

Drag/resize cũng bị lặp giữa pointer và touch, với khác biệt nhỏ về nguồn sự kiện. Fallback output `1920x1080` lặp ở ba tệp. Đây là loại trùng lặp nguy hiểm vì sửa một chỗ dễ bỏ sót.

#### P1-05. Vòng lặp render bị nhân đôi

`backends/winit.rs` và `backends/udev/render.rs` có cùng trình tự: chờ frame, lấy kích thước, kiểm tra screenshot request, dựng element, render, commit. Sự khác biệt chủ yếu là cách lấy surface và DRM. Khi thêm backend mới hoặc thay đổi chính sách frame, phải sửa hai nơi.

#### P1-06. Config I/O và watcher trong adapter

`keymap/config.rs` vừa đọc file, vừa export default, vừa parse TOML, vừa spawn thread watcher. `pointer/config.rs` và `touchpad/config.rs` đọc cấu hình qua `services` trong adapter. Điều này vi phạm nguyên tắc adapter chỉ dịch sự kiện; đồng thời làm vòng đời watcher không được quản lý, dễ tạo thread trùng khi reload hoặc test.

#### P1-07. Panic trong luồng runtime

Các `expect` ở khởi tạo có thể chấp nhận nếu fail-fast có ngữ cảnh, nhưng nhiều `unwrap` nằm trong luồng sự kiện:

- khóa mutex của window state;
- truy cập `toplevel()` khi cửa sổ có thể đang đóng;
- dispatch client;
- xử lý selection/clipboard.

Một client lỗi hoặc một panic thread có thể làm sập toàn bộ compositor. Clean Architecture không chỉ là chia thư mục; nó còn là cô lập lỗi ở biên.

#### P1-08. Hai hệ thống shortcut và bất nhất active window

`keyboard/shortcuts.rs` hardcode nhiều tổ hợp và được gọi trước keymap cấu hình. Vì vậy keymap không thể remap các phím như Super+M, Super+H, Super+Enter. Đồng thời, `execute_action` dùng `space.elements().next_back()` cho maximize/minimize/fullscreen, còn close dùng `active_window()`. Hai cách chọn “cửa sổ active” có thể khác nhau khi có animation đóng hoặc cửa sổ minimized.

Đây là lỗi thiết kế use case: một hành động nghiệp vụ phải có một nguồn xác định cửa sổ đích.

#### P1-09. Hành động cửa sổ dạng chuỗi

`handle_window_action` nhận chuỗi từ topbar/dock/gesture. Chuỗi sai bị bỏ qua bởi nhánh mặc định, không log, không lỗi. Điều này làm lỗi tích hợp khó phát hiện và ngăn exhaustive check của compiler. `common::Action` đã tồn tại nhưng chỉ phủ bốn hành động và không được dùng thống nhất.

#### P1-10 và P1-11. Thiếu CI và thiếu test luồng chính

Không có CI nghĩa là các tuyên bố về kiến trúc, coverage và chất lượng không được thực thi. Không có test luồng chính nghĩa là refactor chỉ có thể dựa vào đọc mã. Hai vấn đề này nhân lên: vừa không có hàng rào, vừa không có lưới an toàn.

---

## 5. Vấn đề đặt tên và tổ chức thư mục

### 5.1. Nguyên tắc đặt tên đề xuất

1. **Tên theo vai trò trước, công nghệ sau.** `adapters/wayland` rõ hơn `adapters/compositor` nếu module chỉ chứa protocol glue; `adapters/desktop` rõ hơn `adapters/shell` cho UI desktop.
2. **`shell` chỉ dùng cho Wayland shell protocol.** Các protocol `xdg_shell`, `layer_shell` nên nằm dưới `adapters/wayland/protocols`; UI desktop không nên dùng lại tên `shell`.
3. **`state` phải có hậu tố mô tả.** `runtime`, `registry`, `input_state`, `overlay_state`, `clipboard_state` tốt hơn `state` đơn độc.
4. **Tránh tên quá chung.** `common`, `types`, `data`, `handle`, `manager`, `util` chỉ nên dùng khi có hậu tố cụ thể.
5. **Một khái niệm, một tên.** Hiện `window` được dùng cho ít nhất bốn thứ: `smithay::desktop::Window`, cửa sổ đang được quản lý, trạng thái hình ảnh, và metadata. Cần phân biệt bằng tên.
6. **Comment mã nguồn dùng một ngôn ngữ.** Đề xuất tiếng Anh cho comment trong mã; tiếng Việt dành cho tài liệu kiến trúc/người dùng nếu team muốn.
7. **Tên tệp phản ánh nội dung.** `state_data.rs` không cho biết đó là record gì; `types.rs` không cho biết đó là render element type.
8. **Thư mục theo tính năng ở tầng ngoài, theo vai trò ở tầng trong.** Adapter nên chia theo công nghệ/tính năng; domain/application chia theo khái niệm.

### 5.2. Bảng ánh xạ tên và vị trí

| Hiện tại | Vấn đề | Đề xuất | Mức |
| --- | --- | --- | --- |
| `adapters/shell` | Trùng nghĩa với Wayland shell; trộn overlay và app GTK | `adapters/desktop`, tách `overlays` và `apps` | P1 |
| `adapters/compositor` | Vừa là protocol, vừa là runtime, vừa là window manager | `adapters/wayland` cho protocol/runtime; `adapters/window` cho registry | P1 |
| `adapters/compositor/handlers` | “Handler” quá chung | `adapters/wayland/protocols` | P2 |
| `state.rs` + `state/` | Hai kiểu module cùng tồn tại, khó đoán entry point | `state/mod.rs` hoặc đổi cha thành `runtime.rs` + `runtime/` | P2 |
| `state/window/` | “Window” không phân biệt cửa sổ Smithay với record runtime | `adapters/window/` cho registry; `state/window/` chỉ còn nếu là runtime cụ thể | P1 |
| `state/window/state_data.rs` | “Data” vô nghĩa | `adapters/window/record.rs` | P2 |
| `state/window/animation.rs` | Trùng tên với `rendering/animation/window.rs` | `adapters/window/animation_state.rs` và `rendering/animation/window_render.rs` | P2 |
| `state/titlebar.rs` | Luật titlebar nằm ở ba nơi | `application/titlebar/` cho hit-test; `adapters/wayland/hit_test.rs` cho surface; `rendering/titlebar/` cho vẽ | P1 |
| `state/blur.rs` | Tên gợi ý sở hữu blur engine, thực tế chỉ reload config | `adapters/rendering/blur/reload.rs` | P2 |
| `domain/window.rs` | Trộn value object, geometry và cache toàn cục | `domain/window/` gồm metadata/visual state; cache ra adapter | P0 |
| `domain/cursor.rs` | Phụ thuộc Smithay/protocol | `domain/cursor_shape.rs` thuần; mapping ở `adapters/wayland/cursor.rs` | P1 |
| `application/ports.rs` | Trộn port và DTO cấu hình | `application/ports/` + `application/model/gesture_policy.rs` | P1 |
| `application/shortcuts.rs` | Service + global registry trong một tệp | `application/shortcuts/` với service và matcher | P1 |
| `application/windows.rs` | Một tệp cho sizing/placement/restore | `application/windows/` với các module theo luật | P1 |
| `input/common/` | “Common” không nói lên nội dung | `input/coordinates.rs` và `input/transform.rs` | P2 |
| `input/keymap/` + `keyboard/shortcuts.rs` | Hai hệ thống shortcut, tên gần nhau | Một `input/keyboard/dispatch.rs`; parser ở `infrastructure/config/keymap.rs` | P1 |
| `rendering/elements/types.rs` | “Types” quá chung | `rendering/elements/element_types.rs` | P3 |
| `screenshot/editor/handle/` | “Handle” quá chung | `screenshot/editor/actions/` | P2 |
| `adapters/shell/gtk.rs` | Helper GTK đặt cạnh app cụ thể | `adapters/desktop/ui/gtk.rs` | P2 |
| `composition/mod.rs` chứa `run()` | Trộn wiring và lifecycle/autostart | `composition/runtime.rs` + `composition/wiring.rs` | P1 |
| `infrastructure` | Không phải implementation của port; là driver/backend | Giữ tên nhưng định nghĩa rõ là “drivers”; hoặc đổi thành `platform`/`drivers` | P2 |
| `src/tests/...` + `#[path]` | Test không theo chuẩn Rust; đường dẫn dễ vỡ | `#[cfg(test)]` trong tệp nguồn; integration test ở `tests/` | P1 |
| Comment tiếng Việt trong ba tệp | Trộn ngôn ngữ | Chuyển sang tiếng Anh hoặc thống nhất theo quy ước | P3 |

### 5.3. Vấn đề cụ thể của từng tên

**`shell`.** Trong ngữ cảnh Wayland, “shell” là protocol (xdg-shell, layer-shell). Trong crate này, `adapters/shell` lại là nơi chứa lock screen, screenshot editor và clipboard picker. Tên này vừa sai nghĩa kỹ thuật, vừa che giấu việc GTK đang nằm trong compositor. Đổi thành `desktop` và tách `overlays`/`apps` là thay đổi có giá trị cao nhất trong nhóm đổi tên. Lưu ý: `desktop` ở đây là namespace nội bộ của adapter, không phải crate `crates/features/desktop`; nếu muốn tránh nhầm hoàn toàn, dùng `desktop_ui`.

**`state`.** `state.rs` và `state/` cùng tồn tại; bên trong lại có `window`, `switcher`, `launcher`, `blur`, `clipboard`, `metadata`, `active_window`. Đây là một thư mục “mọi thứ còn lại”. Tên `state` không sai, nhưng nó không giúp người đọc biết trạng thái nào thuộc về ai. Nên đổi thành các thành phần có tên miền rõ ràng.

**`window`.** Có bốn nghĩa:

- `smithay::desktop::Window` — handle protocol;
- danh sách cửa sổ đang được quản lý — registry;
- `WindowVisualState` — trạng thái hình ảnh;
- `WindowMetadata` — danh tính ứng dụng.

Khi đọc `state/window/sizing.rs`, không rõ đây là luật của cửa sổ hay thao tác protocol. Đề xuất: `WindowRegistry` cho danh sách, `WindowRecord` cho bản ghi runtime, `WindowVisualState` cho hình ảnh, `WindowMetadata` cho danh tính.

**`application/ports.rs`.** Tên “ports” đúng, nhưng nội dung trộn ba thứ: alias hàm, DTO `GesturePolicy`, và định nghĩa cổng. Nên tách thư mục và đặt DTO vào model.

**`titlebar`.** Ba tầng cùng tên nhưng trách nhiệm khác nhau. Nên đặt hậu tố theo vai trò: `titlebar/layout.rs` (domain), `titlebar/hit_test.rs` (application policy), `titlebar/render.rs` (adapter).

**`common`.** Ở workspace, `libs/common` trộn enum nghiệp vụ (`Action`, `SwipeDirection`) với hằng số thiết kế (`BORDER_MARGIN`, kích thước cursor, thông số blur). Đây là “grab bag” điển hình. Nên tách thành `wm-types` (enum/domain dùng chung) và `ui-tokens` (hằng số giao diện), hoặc ít nhất tách module rõ ràng.

---

## 6. Vấn đề phân chia mã và trùng lặp

### 6.1. Phân rã `BabydraState`

Bảng dưới đây là đề xuất phân rã theo trách nhiệm. Mục tiêu không phải là chia nhỏ tùy tiện, mà là làm cho mỗi thành phần có API hẹp và không cần biết toàn bộ compositor.

| Nhóm trạng thái | Trường hiện tại | Thành phần đích |
| --- | --- | --- |
| Protocol/runtime | `start_time`, `socket_name`, `display_handle`, `loop_signal`, các global Smithay, `seat_state`, `seat`, `data_device_state`, `popups` | `WaylandRuntime` |
| Window registry | `space`, `windows`, active window, metadata, z-order | `WindowRegistry` |
| Input | `move_state`, `resize_state`, `last_titlebar_click`, `trackpad_swipe`, `pointer_location`, `cursor_visible`, `cursor_icon`, `pointer_pressed_buttons`, `touch_slots`, `overlay_touch`, `touch_gesture` | `InputState` |
| Overlay | `alt_tab_switcher`, `launcher` | `OverlayState` (thuộc desktop) |
| Clipboard | `clipboard_pending`, `clipboard_capture`, `clipboard_sender` | `ClipboardState` |
| Render/config | `blur_engine`, `touchpad_natural_scroll`, `mouse_natural_scroll`, `gesture_config`, `load_gesture_policy` | `RenderState` + `ConfigSnapshot` |

Sau khi phân rã, `BabydraState` chỉ nên còn 6–7 trường:

- `wayland: WaylandRuntime`
- `windows: WindowRegistry`
- `input: InputState`
- `overlays: OverlayState`
- `clipboard: ClipboardState`
- `render: RenderState`
- `config: ConfigSnapshot`

Mỗi thành phần nên có `impl` trong tệp của nó, không thêm `impl BabydraState` ở nơi khác. Quy tắc review: một phương thức mới phải thuộc về thành phần sở hữu dữ liệu, không thêm vào facade.

### 6.2. Vấn đề “impl phân mảnh”

Mười khối `impl BabydraState` nằm ở:

- `compositor/state.rs`
- `compositor/state/active_window.rs`
- `compositor/state/blur.rs`
- `compositor/state/clipboard.rs`
- `compositor/state/hit_test.rs`
- `compositor/state/switcher.rs`
- `compositor/state/window/actions.rs`
- `compositor/state/window/animation.rs`
- `compositor/state/window/sizing.rs`
- `input/mod.rs`

Đây là dấu hiệu rõ nhất của việc dữ liệu và hành vi không cùng một chỗ. Cách sửa không phải là gộp tất cả vào một tệp lớn, mà là chuyển hành vi về thành phần sở hữu dữ liệu:

- sizing/animation/actions → `WindowRegistry` hoặc `WindowActions`;
- hit-test → `InputState`/`WindowRegistry` với policy từ application;
- switcher/launcher → `OverlayState`;
- clipboard → `ClipboardState`;
- blur reload → `RenderState`;
- focus/raise → `WindowRegistry` hoặc `FocusService`.

### 6.3. Các cụm trùng lặp

| Logic | Vị trí hiện tại | Đề xuất |
| --- | --- | --- |
| Focus/raise/activate | `button.rs` (2 khối), `focus.rs`, `touch/events.rs`, `switcher.rs` | Một `WindowRegistry::activate` hoặc `FocusService`; adapter chỉ gọi |
| Drag/resize | `pointer/drag.rs`, `touch/drag.rs`, `xdg_shell` move/resize, `button.rs` | Một `DragSession`/`ResizeSession` dùng chung; khác biệt chỉ ở nguồn sự kiện |
| Hit-test titlebar | `state/titlebar.rs`, `domain/titlebar.rs`, `rendering/titlebar/` | Tách layout/hit-test thuần, render riêng |
| Output size fallback | `pointer/motion.rs`, `pointer/button.rs`, `touch/events.rs` | Một helper `output_logical_size` |
| Vòng lặp render | `backends/winit.rs`, `udev/render.rs` | `backends/render_loop.rs` |
| Config load | `keymap/config.rs`, `pointer/config.rs`, `touchpad/config.rs` | `infrastructure/config` |
| Shortcut matching | `keyboard/shortcuts.rs`, `keymap/matcher.rs` | Một dispatcher, keymap là nguồn dữ liệu |
| Active window | `active_window()`, `space.elements().next_back()`, `find_window_for_surface` | Một `WindowRegistry::active()` |
| Window action mapping | `handle_window_action` nhận chuỗi, `common::Action` | Một enum `WindowAction` có kiểu |
| On commit | `handlers/compositor.rs`, `handlers/xdg_shell.rs` | Một chủ sở hữu duy nhất |
| GTK init | `composition/bootstrap.rs`, `shell/gtk.rs` | Một `desktop::ui::gtk` bootstrap |

### 6.4. Hai hệ thống shortcut

Luồng hiện tại:

- `check_key_shortcut` trong `keyboard/shortcuts.rs` hardcode các tổ hợp và được gọi trước.
- Nếu không khớp, `keymap::matcher::command_for` mới được gọi.
- `keymap/config.rs` nạp danh sách shortcut từ TOML và có watcher reload.

Hệ quả:

- Người dùng không thể remap các phím đã hardcode.
- Hai nơi cùng định nghĩa “phím nào làm gì”; thêm action mới phải sửa cả hai.
- Thứ tự ưu tiên không được tài liệu hóa.
- `KeyboardAction` trong `common` và `Shortcut` trong `domain` là hai mô hình gần nhau nhưng không thống nhất.

Đề xuất: một bảng shortcut duy nhất, trong đó các action hệ thống là binding mặc định có thể bị override bởi keymap người dùng. Nếu có binding không cho phép remap, phải ghi rõ trong tài liệu và mã.

### 6.5. Bất nhất cách xác định cửa sổ active

Ba cách đang tồn tại:

- `active_window()`: lấy cửa sổ cuối trong `windows`, bỏ qua cửa sổ đang đóng animation, fallback sang `space`.
- `space.elements().next_back()`: lấy phần tử cuối của Space, không bỏ qua animation, không đồng bộ với `windows`.
- `find_window_for_surface`: tìm theo protocol surface.

Các hành động khác nhau dùng các cách khác nhau. Ví dụ `CloseActiveWindow` dùng `active_window()`, còn `ToggleMaximize`, `Minimize`, `ToggleFullscreen` dùng `space.elements().next_back()`. Khi có cửa sổ đang đóng hoặc minimized, cửa sổ bị tác động có thể không phải cửa sổ người dùng thấy là active.

Đề xuất: một `WindowRegistry::active()` duy nhất, có test cho trường hợp đang animation đóng, minimized, và nhiều output. Mọi action nhận `WindowId`/handle đã được resolve từ registry, không tự tìm lại.

### 6.6. Xử lý lỗi

Hiện trạng phân bố lỗi không có chiến lược:

- Khởi tạo: `expect` với thông điệp ngắn, chấp nhận fail-fast nhưng thiếu ngữ cảnh.
- Runtime: `unwrap` khi khóa mutex, truy cập `toplevel` không kiểm tra, dispatch client kèm `expect`.
- Best-effort: nhiều `let _ =` không phân biệt “được phép thất bại” và “lỗi bị nuốt”.

Đề xuất chiến lược ba mức:

1. **Khởi tạo:** trả `Result` có ngữ cảnh, fail-fast ở `main`/composition. Không dùng `expect` rải rác trong module con.
2. **Sự kiện runtime:** không panic. Lỗi protocol → log + ngắt client; lỗi khóa → log + degraded state; lỗi cấu hình → giữ snapshot cũ.
3. **Best-effort có chủ đích:** nếu bỏ qua lỗi, phải có comment lý do hoặc log ở mức debug; không dùng `let _ =` im lặng.

Cụ thể:

- Thay việc khóa mutex bằng `unwrap` bằng xử lý poisoned lock có chủ đích.
- Thay việc truy cập `toplevel` không kiểm tra bằng kiểm tra tùy chọn tường minh.
- Thay lời gọi dispatch client kèm `expect` bằng xử lý lỗi ở event loop, cân nhắc dừng compositor có kiểm soát thay vì panic.
- Thêm ranh giới lỗi cho từng client: một client lỗi không được kéo cả compositor xuống.

### 6.7. Chiến lược test

Mục tiêu không phải là đạt 80% bằng mọi giá, mà là có lưới an toàn cho các luồng dễ vỡ:

- **Domain:** shortcut matching, titlebar layout/hit-test, cursor shape mapping, geometry.
- **Application:** gesture intent, clipboard MIME, sizing policy, focus policy, switcher selection, autostart filtering.
- **Adapter:** protocol mapping, input dispatch, config parser, render element construction, blur rules.
- **Integration:** một cửa sổ mở/click/focus/close; maximize/restore; Alt+Tab; screenshot; clipboard; config reload.
- **Architecture:** import graph, không cycle, domain không framework.

Đề xuất bỏ `#[path]` và dùng test module inline hoặc `tests/` integration. Với binary crate, phần lớn unit test nên nằm inline; các test cần biên module có thể chuyển sang `tests/` nếu API được mở đủ.

### 6.8. Tệp lớn và cohesion

Các tệp lớn không nhất thiết xấu, nhưng cần tách theo trách nhiệm:

- `touch/events.rs` 477 dòng: tách dispatch, gesture accumulation, drag/resize, overlay interaction.
- `state/switcher.rs` 404 dòng: tách model selection, animation, GTK icon loading; đưa model sang application/desktop.
- `lock/render.rs` 491 dòng: tách layout, widgets, PAM status, wallpaper/avatar.
- `state/sizing.rs` 333 dòng: tách pure policy (application) và protocol apply (adapter).
- `pointer/button.rs` 311 dòng: tách hit-test, action dispatch, drag start, focus.

Quy tắc: một tệp nên trả lời được “trách nhiệm của tôi là gì” trong một câu. Nếu câu trả lời có “và”, đó là ứng viên tách.

---

## 7. Rủi ro

| ID | Rủi ro | Khả năng | Tác động | Biện pháp giảm thiểu | Ưu tiên |
| --- | --- | --- | --- | --- | --- |
| R-01 | Chu trình adapter tiếp tục sinh ra khi thêm tính năng | Cao | Cao | Architecture test + review checklist; phá vòng theo lộ trình | P0 |
| R-02 | God state gây borrow-checker conflict và merge conflict | Cao | Cao | Phân rã state; API hẹp; cấm thêm `impl BabydraState` ngoài facade | P0 |
| R-03 | `on_commit` hai lần gây configure/state lặp | Trung bình | Cao | Một chủ sở hữu; test hồi quy; comment bất biến | P0 |
| R-04 | Panic do `unwrap`/`expect`/poisoned lock | Cao | Cao | Chiến lược lỗi ba mức; loại `unwrap` khỏi runtime | P0 |
| R-05 | Global cache/service locator gây test flaky và trạng thái cũ | Cao | Trung bình | Bỏ static; inject service; vòng đời rõ ràng | P1 |
| R-06 | GTK/Pixbuf trong state lõi gây coupling và crash lan | Trung bình | Cao | Tách desktop overlay; GTK chỉ ở adapter ngoài | P1 |
| R-07 | Watcher/config reload race, thread trùng | Trung bình | Trung bình | Quản lý watcher trong infra; snapshot bất biến; test reload | P1 |
| R-08 | Không có CI khiến chu trình và coverage trôi | Cao | Cao | CI bắt buộc; gate theo tầng | P1 |
| R-09 | Thiếu test luồng chính khiến refactor gây regression | Cao | Cao | Characterization test trước khi di chuyển | P1 |
| R-10 | Tài liệu tuyên bố sạch gây hiểu sai | Cao | Trung bình | Cập nhật ADR/docs theo hiện trạng; ghi rõ nợ | P2 |
| R-11 | Dependency phình to, supply chain rộng | Trung bình | Trung bình | Xóa `wayland-sys`; tách `animation`/`common`; audit định kỳ | P2 |
| R-12 | Tách crate quá sớm làm tăng chi phí | Trung bình | Cao | Chỉ tách sau khi hết cycle; modular monolith trước | P2 |

Rủi ro lớn nhất không nằm ở từng tệp riêng lẻ, mà ở **vòng phản hồi**: chu trình làm thay đổi lan rộng, god state làm thay đổi khó cô lập, thiếu test làm thay đổi rủi ro, thiếu CI làm lỗi kiến trúc quay lại. Đây là một hệ thống tự củng cố tiêu cực; cần cắt vòng bằng hàng rào tự động trước.

---

## 8. Kiến trúc đích

### 8.1. Nguyên tắc

1. **Một chiều, không chu trình.** `composition → infrastructure → adapters → application → domain`. Trong nội bộ adapter, thứ tự cụ thể là `wayland → desktop → rendering → window → input → application`.
2. **Lõi không biết framework.** `domain` và `application` không import Smithay, GTK, Wayland protocol, `std::fs`, `std::process`, hoặc static mutable.
3. **Quyết định thuộc application, hiệu ứng thuộc adapter.** Application trả về intent/command; adapter thực thi.
4. **State có chủ.** Mỗi trường state thuộc một thành phần; không có god object; không thêm `impl` vào facade.
5. **Port là trait khi có hiệu ứng.** Hàm thuần vẫn có thể là hàm; I/O, process, config, clipboard, notification phải là trait có `Result`.
6. **Lỗi được cô lập ở biên.** Client lỗi không làm sập compositor; config lỗi giữ snapshot cũ; startup lỗi fail-fast có ngữ cảnh.
7. **Một nguồn luật.** Shortcut, active window, action mapping, output size, hit-test chỉ có một nguồn.
8. **Hàng rào tự động.** Kiến trúc không thể chỉ dựa vào review.

### 8.2. Cây module mục tiêu

Đây là cấu trúc định hướng, không phải yêu cầu di chuyển tất cả trong một lần:

- `src/domain/`
  - `window/` — `WindowMetadata`, `WindowVisualState`, kiểu hình học thuần
  - `placement.rs` — luật vị trí/restore
  - `titlebar.rs` — layout và hit-test thuần
  - `shortcuts.rs` — mô hình shortcut, matching thuần
  - `cursor_shape.rs` — enum hình dạng con trỏ thuần
  - `keysyms.rs` — bảng keysym
- `src/application/`
  - `ports/` — trait cho shortcut repository, command launcher, gesture policy, active-window sink, clipboard/notification
  - `model/` — `GesturePolicy`, command/intent, error
  - `windows/` — sizing, placement, focus, actions
  - `switcher/` — selection, ordering, commit/cancel
  - `clipboard.rs` — chọn MIME, chính sách capture
  - `gestures.rs` — intent từ gesture
  - `shortcuts/` — service, matcher, reload
  - `autostart.rs` — lọc và quản lý process autostart
  - `titlebar.rs` — chính sách hit-test/action
- `src/adapters/`
  - `wayland/`
    - `protocols/` — compositor, xdg_shell, xdg_decoration, layer_shell
    - `runtime.rs` — facade protocol/runtime, event loop state
    - `seat.rs` — keyboard/pointer focus
    - `clipboard.rs` — data device/selection
    - `cursor.rs` — bitmap và mapping sang Smithay
    - `hit_test.rs` — surface/occlusion
  - `window/`
    - `registry.rs` — danh sách, z-order, active
    - `metadata.rs` — cache metadata theo surface
    - `record.rs` — dữ liệu runtime của một cửa sổ
    - `animation_state.rs` — trạng thái animation
    - `geometry.rs` — áp dụng hình học qua protocol
  - `input/`
    - `keyboard/` — dispatch, focus
    - `pointer/` — button, motion, drag
    - `touch/` — events, gestures
    - `touchpad/` — scroll, swipe, gestures
    - `interaction/` — drag/resize/activate dùng chung
    - `coordinates.rs` — transform tuyệt đối
  - `rendering/`
    - `elements/` — window, layer, element type
    - `animation/` — render interpolation
    - `blur/` — engine, config, GL
    - `titlebar/` — vẽ titlebar
    - `buffer.rs`, `frame.rs`
  - `desktop/`
    - `overlays/switcher/` — view, element, draw, state
    - `overlays/launcher/` — buffer, draw, state
    - `apps/lock/`
    - `apps/screenshot/`
    - `apps/clipboard/`
    - `ui/gtk.rs` — helper GTK dùng chung
- `src/infrastructure/`
  - `backends/` — `render_loop.rs`, `winit.rs`, `udev/`
  - `config/` — `input.rs`, `keymap.rs`, `gesture.rs`, `watcher.rs`
  - `process.rs` — launch command
  - `platform/` — brand, môi trường
- `src/composition/`
  - `runtime.rs` — vòng đời event loop, shutdown
  - `wiring.rs` — xây dependency
  - `bootstrap.rs` — tracing, env, GTK
  - `commands.rs` — subcommand
  - `input_method.rs` — biến môi trường bộ gõ
- `src/main.rs`
- Test: inline `#[cfg(test)]` cho unit; `tests/` cho integration/smoke.

### 8.3. Quy tắc phụ thuộc trong adapter

Mục tiêu là loại bỏ ba chu trình bằng cách tách “runtime facade” khỏi “thành phần dùng chung”:

- `adapters/window` là lớp thấp, chỉ phụ thuộc `application`/`domain`; không biết input, rendering, desktop, wayland.
- `adapters/input` phụ thuộc `window` và `application`; không biết `desktop`, không biết `wayland` runtime.
- `adapters/rendering` phụ thuộc `window` (visual state/metadata) và `domain`; không biết `wayland`, không gọi ngược compositor.
- `adapters/desktop` phụ thuộc `application`, `window`, `rendering`; không biết `wayland` runtime.
- `adapters/wayland` là facade cao nhất, phụ thuộc `window`, `input`, `rendering`, `desktop`; không bị các module kia import ngược.
- `infrastructure` phụ thuộc `application` (port) và có thể phụ thuộc `adapters` để điều khiển runtime; không bị `adapters` import ngược.
- `composition` phụ thuộc tất cả.

Nếu cần một chiều ngược từ `input`/`desktop` lên `wayland`, hãy dùng trait/command trong `application` hoặc một interface hẹp trong `adapters/window`, không import facade.

### 8.4. Phân rã state và quyền sở hữu

Sáu thành phần đã nêu ở 6.1 có thể triển khai theo hướng:

- `WaylandRuntime` sở hữu protocol global, seat, display, loop signal, socket. Nó là nơi duy nhất implement `CompositorHandler`, `XdgShellHandler`, `LayerShellHandler`, v.v.
- `WindowRegistry` sở hữu `Space`, danh sách `Window`, metadata, active window, z-order. Nó cung cấp truy vấn `active`, `find_by_surface`, `raise`, `activate`.
- `InputState` sở hữu pointer/touch/gesture/move/resize/last click. Nó không tự focus; nó trả intent cho `WindowRegistry`/application.
- `OverlayState` sở hữu switcher/launcher, nhưng nên chuyển sang `adapters/desktop`. Runtime chỉ giữ handle hoặc trạng thái “overlay đang mở”.
- `ClipboardState` sở hữu pending/capture/sender; adapter Wayland xử lý selection, service application chọn MIME.
- `RenderState` sở hữu blur engine, natural scroll, render config; renderer nhận snapshot, không truy vấn ngược.

Quy tắc: không thành phần nào giữ con trỏ đến `BabydraState`. Nếu cần nhiều thành phần, truyền từng tham chiếu hẹp hoặc một context chỉ đọc.

### 8.5. Port và dependency injection

Các port nên là trait khi có hiệu ứng hoặc cần thay thế:

- `ShortcutRepository`: tải, reload, theo dõi thay đổi; trả `Result`.
- `CommandLauncher`: chạy lệnh; trả `Result` hoặc handle.
- `GesturePolicyProvider`: snapshot policy.
- `ActiveWindowSink`: công bố cửa sổ active cho topbar/dock.
- `ClipboardService`: capture/selection/history.
- `NotificationService`: thông báo kết quả screenshot/lock.
- `ConfigStore`: đọc/ghi cấu hình và watcher (infrastructure implement).

Không cần trait hóa mọi hàm thuần. `maximize_target`, `scrub_index`, `preferred_mime` nên tiếp tục là hàm thuần, dễ test.

Composition tạo implementation cụ thể một lần và truyền vào service. Không có `OnceLock`, không có `get()`, không có global mutable.

### 8.6. GTK và ứng dụng desktop

GTK chỉ nên xuất hiện ở:

- `adapters/desktop` cho overlay/app;
- `infrastructure/platform` cho khởi tạo toolkit;
- `composition/bootstrap` cho lifecycle.

Không đưa `Pixbuf`, widget, hay `glib::MainContext` vào `WindowRegistry`, `InputState`, `WaylandRuntime`. Nếu switcher cần icon, desktop overlay sở hữu pixbuf/texture; runtime chỉ giữ danh sách cửa sổ và selection.

Dài hạn, lock/screenshot/clipboard editor nên là process hoặc crate riêng, giao tiếp với compositor qua IPC/portal. Điều này giảm rủi ro GTK crash kéo compositor và cho phép phát triển UI độc lập. Không cần làm ngay; chỉ cần tách module để chuẩn bị.

### 8.7. Tách crate (tuỳ chọn, sau khi hết cycle)

Sau khi kiến trúc module ổn định, có thể tách:

- `babydra-wm-core`: domain + application, không Smithay/GTK.
- `babydra-wm-wayland`: adapter Wayland + window/input + backend.
- `babydra-wm-render`: render element/blur/animation.
- `babydra-desktop`: overlay + ứng dụng GTK.
- `babydra-window-manager`: composition, binary.

Lợi ích: compiler enforcement, test độc lập, build song song, binary nhỏ hơn. Chi phí: visibility, versioning, build phức tạp. Chỉ tách sau khi ba chu trình đã biến mất và test đủ mạnh; tách sớm sẽ biến nợ kiến trúc thành nợ build.

### 8.8. Test và CI mục tiêu

- CI bắt buộc: `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test`, `cargo deny`, architecture test.
- Architecture test kiểm tra:
  - không import ra ngoài chiều cho phép;
  - không cycle;
  - `domain` không Smithay/GTK/fs/process;
  - `application` không static mutable/service locator.
- Coverage gate theo tầng, bắt đầu thấp và tăng dần; không đặt mục tiêu 80% cho adapter ngay lập tức.
- Smoke matrix giữ nguyên, nhưng thêm checklist chạy trước merge cho các luồng P0.

---

## 9. Kế hoạch di chuyển và đổi tên chi tiết

Các bảng dưới đây là đề xuất đích danh. Thứ tự thực hiện nên theo lộ trình ở mục 10, không nhất thiết theo thứ tự bảng. Mỗi bước phải biên dịch và test độc lập; không trộn di chuyển với thay đổi hành vi.

### 9.1. Domain và application

| ID | Hiện tại | Đích | Hành động | Lý do | Pha |
| --- | --- | --- | --- | --- | --- |
| D01 | `domain/cursor.rs` | `domain/cursor_shape.rs` + `adapters/wayland/cursor.rs` | Tách enum thuần và mapping Smithay | Loại protocol khỏi domain | 3 |
| D02 | `domain/window.rs` | `domain/window/` gồm metadata, visual state, geometry thuần | Tách tệp; bỏ cache static | Domain không sở hữu cache/vòng đời | 3 |
| D03 | `application/ports.rs` | `application/ports/` + `application/model/gesture_policy.rs` | Tách port và DTO | Port rõ nghĩa, DTO đúng chỗ | 3 |
| D04 | `application/shortcuts.rs` | `application/shortcuts/` gồm service, matcher, reload | Tách tệp; bỏ `OnceLock` | Loại service locator, inject dependency | 3 |
| D05 | `application/windows.rs` | `application/windows/` gồm sizing, placement, focus, actions | Tách theo luật | Application sở hữu quyết định cửa sổ | 3 |
| D06 | `application/switcher.rs` | `application/switcher/` gồm selection, ordering | Tách theo luật | Tách luật chọn khỏi UI/animation | 3 |
| D07 | `composition/run()` (autostart) | `application/autostart.rs` + port process | Di chuyển use case | Autostart là nghiệp vụ, không phải wiring | 3 |
| D08 | `state/titlebar.rs` (phần policy) | `application/titlebar.rs` | Tách policy khỏi hit-test protocol | Một nguồn luật titlebar | 3 |
| D09 | `application/gestures.rs`, `clipboard.rs` | Giữ nguyên, có thể thành thư mục khi lớn | Không di chuyển ngay | Tránh churn không cần thiết | — |

### 9.2. Adapter runtime, protocol và window

| ID | Hiện tại | Đích | Hành động | Lý do | Pha |
| --- | --- | --- | --- | --- | --- |
| A01 | `adapters/compositor/handlers/` | `adapters/wayland/protocols/` | Đổi tên thư mục | “Handler” quá chung; đây là protocol | 2 |
| A02 | `adapters/compositor/state.rs` | `adapters/wayland/runtime.rs` + các thành phần state | Tách facade và state | Phá god object/chu trình | 2 |
| A03 | `adapters/compositor/state/window/` | `adapters/window/` | Di chuyển registry/record | State cửa sổ không thuộc protocol | 2 |
| A04 | `adapters/compositor/state/window/sizing.rs` | `application/windows/sizing.rs` + `adapters/window/geometry.rs` | Tách policy và apply protocol | Quyết định thuộc application | 2–3 |
| A05 | `adapters/compositor/state/window/actions.rs` | `application/windows/actions.rs` + adapter executor | Tách policy; dùng enum `WindowAction` | Bỏ stringly typed, exhaustive | 1–3 |
| A06 | `adapters/compositor/state/window/state_data.rs` | `adapters/window/record.rs` | Đổi tên/di chuyển | Tên “data” vô nghĩa | 2 |
| A07 | `adapters/compositor/state/window/animation.rs` | `adapters/window/animation_state.rs` | Đổi tên; đổi `rendering/animation/window.rs` thành `window_render.rs` | Phân biệt state và render | 2 |
| A08 | `adapters/compositor/state/titlebar.rs` | `application/titlebar.rs` + `adapters/wayland/hit_test.rs` | Tách policy và surface lookup | Ba nơi titlebar hiện tại quá mơ hồ | 2–3 |
| A09 | `adapters/compositor/state/hit_test.rs` | `adapters/wayland/hit_test.rs` | Di chuyển/đổi tên | Hit-test protocol thuộc Wayland adapter | 2 |
| A10 | `adapters/compositor/state/metadata.rs` | `adapters/window/metadata.rs` | Di chuyển cache | Cache không thuộc domain/compositor | 2 |
| A11 | `adapters/compositor/state/active_window.rs` | `adapters/window/active.rs` + `ActiveWindowSink` | Di chuyển; inject sink | Tách hiệu ứng công bố active window | 2 |
| A12 | `adapters/compositor/state/blur.rs` | `adapters/rendering/blur/reload.rs` | Di chuyển/đổi tên | Đúng chủ sở hữu blur | 2 |
| A13 | `adapters/compositor/state/switcher.rs` | `adapters/desktop/overlays/switcher/state.rs` + application selection | Tách model/UI | Loại GTK/overlay khỏi state lõi | 2–4 |
| A14 | `adapters/compositor/state/launcher.rs` | `adapters/desktop/overlays/launcher/state.rs` | Di chuyển state/buffer | Launcher là desktop overlay | 2–4 |
| A15 | `adapters/compositor/state/clipboard.rs` | `adapters/wayland/clipboard.rs` | Di chuyển | Clipboard protocol thuộc Wayland | 2 |
| A16 | `adapters/compositor/cursor/data.rs` | `adapters/wayland/cursor/bitmap.rs` | Đổi tên/di chuyển | “Data” không mô tả bitmap | 2 |

### 9.3. Input và cấu hình

| ID | Hiện tại | Đích | Hành động | Lý do | Pha |
| --- | --- | --- | --- | --- | --- |
| I01 | `adapters/input/keyboard/shortcuts.rs` | `adapters/input/keyboard/dispatch.rs` | Đổi tên; hợp nhất với keymap | Một đường dispatch duy nhất | 1–3 |
| I02 | `adapters/input/keymap/` | `infrastructure/config/keymap.rs` + `application/shortcuts/matcher.rs` + `resources/keymap.toml` | Tách parser/matcher/resource | Config I/O không thuộc adapter input | 2–3 |
| I03 | `adapters/input/pointer/config.rs`, `touchpad/config.rs` | `infrastructure/config/input.rs` | Gộp và di chuyển | Một nơi đọc cấu hình input | 2 |
| I04 | `adapters/input/pointer/drag.rs`, `touch/drag.rs` | `adapters/input/interaction/drag.rs` | Gộp | Một state machine drag/resize | 2 |
| I05 | Focus/raise trong `button.rs`, `touch/events.rs`, `focus.rs`, `switcher.rs` | `adapters/input/interaction/activate.rs` + `WindowRegistry` | Trích xuất | Loại trùng lặp, một nguồn active | 2 |
| I06 | `adapters/input/common/` | `adapters/input/coordinates.rs` + `transform.rs` | Đổi tên/tách | “Common” không mô tả nội dung | 2 |
| I07 | `adapters/input/touch/events.rs` | `touch/dispatch.rs` + `touch/gesture_state.rs` | Tách tệp 477 dòng | Giảm trách nhiệm | 2 |
| I08 | `adapters/input/pointer/button.rs` | `pointer/button.rs` + `interaction/action_dispatch.rs` | Tách hit-test/action/drag | Bỏ lặp focus/raise | 2 |

### 9.4. Desktop, infrastructure, composition và test

| ID | Hiện tại | Đích | Hành động | Lý do | Pha |
| --- | --- | --- | --- | --- | --- |
| U01 | `adapters/shell/` | `adapters/desktop/` | Đổi tên thư mục | “Shell” trùng nghĩa protocol | 1–4 |
| U02 | `adapters/shell/gtk.rs` | `adapters/desktop/ui/gtk.rs` | Di chuyển/đổi tên | Helper GTK dùng chung | 4 |
| U03 | `shell/lock`, `shell/screenshot`, `shell/clipboard` | `adapters/desktop/apps/{lock,screenshot,clipboard}` | Di chuyển | Ứng dụng GTK khác overlay | 4 |
| U04 | `shell/switcher` + state switcher | `adapters/desktop/overlays/switcher/` | Gộp | Một nơi sở hữu UI switcher | 4 |
| U05 | `shell/launcher` + state launcher | `adapters/desktop/overlays/launcher/` | Gộp | Một nơi sở hữu UI launcher | 4 |
| U06 | `Pixbuf` trong `state/switcher.rs` | Desktop overlay state/texture | Di chuyển | GTK không thuộc state lõi | 4 |
| C01 | `composition/run()` | `composition/runtime.rs` + `composition/wiring.rs` | Tách lifecycle/wiring | Composition chỉ lắp ghép | 2 |
| C02 | `backends/winit.rs`, `udev/render.rs` | `backends/render_loop.rs` | Trích xuất vòng lặp chung | Loại nhân đôi backend | 4 |
| C03 | Backend gọi trực tiếp screenshot shell | Port `ScreenshotService` | Đảo phụ thuộc | Backend không biết UI | 3–4 |
| C04 | `wayland-sys`; `animation` kéo GTK; `common` grab bag | Xóa dependency; tách crate/module | Dọn dependency | Giảm build/supply chain | 4 |
| C05 | `src/tests` + `#[path]` | Inline test + `tests/` | Di chuyển test | Theo chuẩn Rust, coverage đúng | 0–1 |
| C06 | ADR 0003/docs mô tả sai hiện trạng | Cập nhật docs + trạng thái nợ | Sửa tài liệu | Tránh niềm tin sai | 0 |
| C07 | `#[allow(unused_imports)]`, comment trộn ngôn ngữ | Xóa allow; thống nhất comment | Dọn dẹp | Minh bạch, nhất quán | 1 |
| C08 | Fallback `1920x1080` lặp ba nơi | `adapters/window/output_size.rs` | Gộp helper | Một hành vi fallback | 2 |
| C09 | Tệp >400 dòng ở shell/input/state | Các module nhỏ theo trách nhiệm | Tách tệp | Giảm tải nhận thức | 3–4 |

---

## 10. Lộ trình thực hiện

Ước lượng dưới đây là tương đối cho 1–2 kỹ sư đã quen mã nguồn; thời gian thực tế phụ thuộc mức độ song song với tính năng.

### Pha 0 — Hàng rào an toàn (khoảng 1 tuần)

Mục tiêu: biết khi nào mình làm vỡ kiến trúc, trước khi di chuyển bất cứ thứ gì.

Việc cần làm:

- Thêm CI: fmt, clippy `-D warnings`, test, deny.
- Viết architecture test đơn giản: cấm import ra ngoài chiều cho phép; phát hiện cycle.
- Thêm characterization test cho:
  - `on_commit` của toplevel;
  - sizing maximize/restore/fullscreen;
  - focus/raise;
  - button/touch dispatch;
  - shortcut ưu tiên hardcode vs keymap;
  - config reload.
- Ghi baseline số liệu: cycle, unwrap, LOC theo tầng.
- Cập nhật ADR/docs để mô tả đúng hiện trạng, không tuyên bố đã sạch.

Tiêu chí thoát: CI chạy trên PR; architecture test fail nếu thêm một import sai; test mới chạy ổn định.

### Pha 1 — Sửa lỗi P0 và dọn nhanh (1–2 tuần)

Mục tiêu: chặn rủi ro đúng đắn và giảm nhiễu trước khi tái cấu trúc.

Việc cần làm:

- Sửa `on_commit` hai lần.
- Thay `handle_window_action` nhận chuỗi bằng enum có kiểu; log hành động không hợp lệ.
- Hợp nhất cách xác định active window.
- Loại `#[allow(unused_imports)]`, xóa `wayland-sys` nếu không dùng.
- Thay các `unwrap` nguy hiểm trong runtime bằng xử lý lỗi; ít nhất ở khóa mutex và `toplevel()`.
- Đổi tên `adapters/shell` → `adapters/desktop` ở mức thư mục, chưa cần tách app.
- Chuyển test khỏi `#[path]` nếu có thể làm từng phần.

Tiêu chí thoát: không còn double commit; không còn panic đã biết trong luồng click/close; CI xanh; tên `desktop` được dùng thống nhất.

### Pha 2 — Phá chu trình và tách state (3–4 tuần)

Mục tiêu: biến ba chu trình thành quan hệ một chiều; `BabydraState` thành facade.

Thứ tự đề xuất:

1. Tách `WindowRegistry` khỏi `BabydraState`; di chuyển metadata cache khỏi domain.
2. Tách `InputState`; đưa move/resize/pointer/touch/gesture vào đó.
3. Tách `OverlayState`; chuyển switcher/launcher state sang desktop.
4. Tách `ClipboardState`; chuyển selection protocol sang Wayland adapter.
5. Tách `WaylandRuntime` facade; chuyển các `impl` protocol về đúng thành phần.
6. Sửa `rendering` để nhận metadata/visual snapshot, không gọi ngược compositor.
7. Sửa `input` để không import `shell`; dùng intent/command hoặc interface overlay hẹp.
8. Thêm architecture test cho từng cặp sau khi phá.

Tiêu chí thoát: 0 cycle; `BabydraState` ≤ 7 trường; `rendering` không import `compositor`; `input` không import `shell`; test vẫn xanh.

### Pha 3 — Làm sạch lõi (3–4 tuần)

Mục tiêu: domain/application thực sự thuần và sở hữu quyết định.

Việc cần làm:

- Loại Smithay/GTK khỏi domain; thay bằng kiểu thuần.
- Đưa sizing, focus, switcher, placement, autostart từ adapter vào application.
- Thay port hàm bằng trait có `Result`; thêm fake cho test.
- Bỏ `OnceLock` và service locator; inject `ShortcutService`.
- Hợp nhất hai hệ thống shortcut.
- Chuyển config I/O/watcher về infrastructure.
- Thêm test application với fake port.

Tiêu chí thoát: domain/application không import framework; application có use case rõ ràng; shortcut có một đường; config chỉ I/O ở infra.

### Pha 4 — Tách desktop/backend và dọn tên (4–6 tuần)

Mục tiêu: GTK ra khỏi lõi, backend không lặp, tên phản ánh trách nhiệm.

Việc cần làm:

- Tách `desktop/overlays` và `desktop/apps`.
- Đưa `Pixbuf`/widget khỏi state lõi.
- Tách `backends/render_loop.rs` dùng chung.
- Đưa screenshot request thành port.
- Tách các tệp lớn: touch/events, switcher, lock/render, sizing, button.
- Cân nhắc tách crate nếu cycle đã hết và test đủ mạnh.
- Dọn dependency: `animation`/`common`.

Tiêu chí thoát: GTK chỉ ở desktop/platform; backend không gọi UI; các tệp lớn đã tách; smoke matrix xanh trên Winit và Udev.

### Pha 5 — Duy trì (liên tục)

- Architecture test là gate bắt buộc.
- Review checklist có mục “import có đúng chiều không?”.
- Đo coverage theo tầng, không chỉ tổng.
- Mỗi PR refactor nhỏ, không trộn hành vi.
- Cập nhật ADR khi có quyết định mới; không để docs lệch.

---

## 11. Tiêu chí nghiệm thu

### 11.1. Checklist kiến trúc

- [ ] Không còn chu trình giữa các module.
- [ ] `domain` không import Smithay, GTK, Wayland protocol, `std::fs`, `std::process`.
- [ ] `application` không có static mutable, không service locator, không `OnceLock`.
- [ ] `BabydraState` chỉ còn vai trò facade với ≤ 7 thành phần.
- [ ] Mỗi thành phần state có `impl` trong tệp của nó; không thêm `impl` vào facade.
- [ ] `rendering` không import `compositor`/`wayland`.
- [ ] `input` không import `desktop`/`shell`.
- [ ] `desktop` không import `wayland` runtime.
- [ ] Port có hiệu ứng là trait, có `Result` và fake test.
- [ ] Chỉ còn một hệ thống shortcut.
- [ ] Chỉ còn một cách xác định active window.
- [ ] Chỉ còn một nơi gọi `on_commit` cho toplevel.
- [ ] Config I/O và watcher chỉ nằm trong `infrastructure/config`.
- [ ] GTK/Pixbuf không nằm trong state lõi.

### 11.2. Checklist chất lượng

- [ ] CI chạy fmt, clippy `-D warnings`, test, deny.
- [ ] Architecture test fail khi import sai chiều.
- [ ] Không còn `unwrap`/`expect` trong luồng sự kiện runtime (có thể còn ở khởi tạo có ngữ cảnh).
- [ ] Không còn `let _ =` không có lý do.
- [ ] Test cho sizing, focus, switcher, input dispatch, config reload, double commit.
- [ ] Smoke matrix Winit/Udev được chạy trước release.
- [ ] Coverage theo tầng có ngưỡng và được báo cáo.
- [ ] Docs/ADR khớp hiện trạng.

### 11.3. Checklist đặt tên

- [ ] `shell` chỉ dùng cho Wayland shell protocol.
- [ ] Không còn `state` đơn độc mà không có hậu tố.
- [ ] Không còn `common`, `types`, `data`, `handle` đơn độc trong tên module mới.
- [ ] `window` được phân biệt: registry, record, metadata, visual state.
- [ ] `titlebar` có hậu tố theo vai trò: layout, hit-test, render.
- [ ] Comment mã nguồn thống nhất một ngôn ngữ.
- [ ] Không còn `#[allow(unused_imports)]`.

---

## 12. Kết luận

`crates/window-manager` không phải là một codebase tồi; nó có tính năng phong phú, tài liệu tốt, `cargo check`/`clippy` sạch và một số test có giá trị. Vấn đề là **khoảng cách giữa mô hình kiến trúc được tuyên bố và cấu trúc thực tế**. Việc đổi tên thư mục sang `domain/application/adapters/infrastructure` đã tạo ra vỏ bọc Clean Architecture, nhưng không tạo ra ranh giới. Ba chu trình, god state 39 trường, application 409 dòng và 85% mã nằm trong adapter là bằng chứng định lượng.

Nếu tiếp tục thêm tính năng trên cấu trúc hiện tại, chi phí sẽ tăng theo cấp số nhân: mỗi thay đổi protocol có thể chạm input/rendering/shell; mỗi thay đổi UI có thể chạm state lõi; mỗi refactor không có test để bảo vệ. Ngược lại, nếu đầu tư 2–3 tháng có kỷ luật, có thể đưa crate về một kiến trúc modular monolith sạch, với lõi thuần, adapter một chiều, và hàng rào tự động.

Ba ưu tiên nếu chỉ chọn ba việc:

1. **Dựng CI + architecture test + characterization test** — không có hàng rào thì mọi nỗ lực sau đó sẽ trôi.
2. **Phá chu trình bằng `WindowRegistry`/`InputState`/`OverlayState`** — đây là gốc của phần lớn vấn đề còn lại.
3. **Đưa quyết định từ adapter vào application** — biến Clean Architecture từ hình thức thành vận hành.

Làm đúng thứ tự này, crate có thể tiến hóa mà không cần viết lại; làm ngược lại, nhiều khả năng sẽ tạo ra một cuộc “đại refactor” thứ hai với cùng kết quả.

---

## 13. Trạng thái triển khai

Mục này được cập nhật trong quá trình thực hiện kế hoạch. Các mục đã xong được commit riêng biệt theo từng phần công việc.

### 13.1. Đã thực hiện

- Hàng rào an toàn: CI chạy fmt, clippy, test, deny và architecture test.
- Report này được làm mới để phản ánh đúng hiện trạng 123 tệp nguồn.
- Sửa `on_commit` bị gọi hai lần; chuẩn hóa action cửa sổ thành enum có kiểu; thống nhất cách chọn cửa sổ active.
- Loại `unwrap` khi khóa mutex trạng thái cửa sổ; bỏ `allow(unused_imports)`; xóa dependency `wayland-sys`.
- Đổi `adapters/shell` thành `adapters/desktop`; chuyển test từ `src/tests` + `#[path]` về module inline.
- Đưa cache metadata ra khỏi `domain` sang `adapters/window`; đưa ánh xạ con trỏ ra khỏi `domain`.
- Chuyển state switcher sang `adapters/desktop`; chuyển cấu hình keymap/pointer/touchpad vào `infrastructure/config`.
- Tách input khỏi trạng thái compositor bằng trait `InputContext` và các state type trung lập; gom các bước dùng chung của hai vòng lặp render vào `infrastructure/backends/render_loop` (kèm sửa lỗi path/scope trong backend Udev).
- Đưa dịch vụ shortcut ra khỏi `OnceLock` toàn cục: `ShortcutService` được inject qua composition; mọi action desktop từ input đi qua `InputContext`, xóa cạnh `input → desktop` và thu baseline kiến trúc từ 7 xuống 6 cạnh (không còn chu trình).
- Làm sạch lõi: `domain/geometry.rs` cung cấp `Point`/`Size`/`Rectangle` thuần; `domain` và `application` không còn import Smithay/GTK (`cargo` + architecture test xác nhận).
- Đưa autostart thành use case trong `application/autostart.rs` với port `AutostartLauncher` và fake test; tách `composition` thành coordinator + `wiring.rs` + `runtime.rs`; lỗi đăng ký clipboard channel giờ trả `Result` có ngữ cảnh thay vì `expect`.
- Tách helper animation phụ thuộc GTK sang crate `animation-gtk`; `animation` và `window-manager` không còn kéo GTK qua dependency này.
- Gộp fallback cấu hình vào `infrastructure/config`, bỏ re-export adapter→infrastructure, xóa `#[allow(unused_imports)]`, thống nhất comment mã nguồn sang tiếng Anh.
- Mở rộng architecture gate: cấm framework/service locator trong lõi, cấm `allow(unused_imports)`, đảm bảo `on_commit` chỉ có một chủ sở hữu. Hiện có 6 test kiến trúc, 61 unit test.
- Cập nhật `docs/architecture.md`, `docs/testing.md` và ADR 0003/0004 cho khớp hiện trạng.

### 13.2. Đang thực hiện hoặc còn lại

- Phân rã tiếp `BabydraState` (39 trường) thành facade ~7 thành phần: `ClipboardState` đã tách; đang tách `RenderState`/`ConfigSnapshot`, sau đó đến `InputState`, `OverlayState`, `WaylandRuntime`, `WindowRegistry`.
- Chuyển các port còn lại (`LaunchCommand`, `LoadShortcuts`, `LoadGesturePolicy`) từ alias hàm sang trait có `Result` và fake test.
- Giảm `unwrap`/`expect` trong luồng sự kiện runtime (đặc biệt khóa mutex và truy cập seat/toplevel); đưa screenshot request thành port để backend không gọi trực tiếp UI.
- Thêm characterization test cho sizing, focus, input dispatch, switcher và config reload; thiết lập coverage gate theo tầng trong CI.
- Tách các tệp lớn còn lại (`touch/events.rs`, `lock/render.rs`, `switcher/element.rs`, `blur/gl/pipeline.rs`) theo trách nhiệm.
- Tách `common` thành nhóm type/enum và design token; cân nhắc tách crate cuối cùng sau khi hết nợ kiến trúc.
