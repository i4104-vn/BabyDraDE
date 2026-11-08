# Windows 11 Default Fonts

Bộ sưu tập các font chuẩn trích xuất trực tiếp từ Windows 11 phục vụ dựng giao diện (UI), thanh trạng thái (Topbar), tiêu đề cửa sổ (Titlebar) và hiển thị icon/glyph trong BabyDraDE.

## Cấu trúc thư mục

### 1. `ui/` — Giao diện chính (Fluent Design)
- **`SegUIVar.ttf` (Segoe UI Variable)**: Font giao diện mặc định chủ đạo của Windows 11 (Taskbar, Start Menu, Settings, File Explorer). Tối ưu hóa dynamic optical sizing cho cả màn hình độ phân giải cao và chữ kích thước nhỏ.
- **`seguisb.ttf` (Segoe UI Semibold)**: Dùng cho tiêu đề cửa sổ, header các widget, tên app.
- **`segoeui.ttf` (Segoe UI Regular)**: Chữ nội dung thông thường, nhãn, menu item.
- **`segoeuib.ttf` (Segoe UI Bold)**: Chữ đậm nhấn mạnh.
- **`segoeuisl.ttf` / `segoeuil.ttf`**: Chữ mỏng/phụ (Subtitles, secondary text).

### 2. `icons/` — Biểu tượng hệ thống (Glyph Icons)
- **`SegoeIcons.ttf` (Segoe Fluent Icons)**: Bộ icon vector chính thức của Windows 11 (dùng cho icon pin, wifi, âm lượng, cài đặt, cửa sổ...).
  - Một số mã Unicode thông dụng:
    - `\u{E701}` / `\u{E708}`: Wi-Fi
    - `\u{E83F}` / `\u{EBAE}`: Battery
    - `\u{E767}` / `\u{E74F}`: Volume / Mute
    - `\u{E706}`: Brightness
    - `\u{E713}`: Settings
    - `\u{E721}`: Search
    - `\u{E7E8}`: Power / Shutdown
    - `\u{E921}` / `\u{E922}` / `\u{E923}`: Minimize / Maximize / Restore
    - `\u{E8BB}`: Close
- **`segmdl2.ttf` (Segoe MDL2 Assets)**: Bộ icon UWP kế thừa.

### 3. `mono/` — Phông đơn cách (Terminal & Code)
- **`consola.ttf` / `consolab.ttf` / `consolai.ttf` / `consolaz.ttf` (Consolas)**: Font monospace tiêu chuẩn của Windows dành cho cửa sổ dòng lệnh (Terminal, Command Prompt) và trình soạn thảo (Notepad).

### 4. `emoji/` — Biểu tượng cảm xúc
- **`seguiemj.ttf` (Segoe UI Emoji)**: Bảng ký tự Emoji đầy đủ của Windows.
