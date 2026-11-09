# BabyDra Design System & Style Guide

Tài liệu trích xuất toàn bộ ngôn ngữ thiết kế, hệ thống token, quy chuẩn giao diện và kiến trúc CSS từ [`babydra-core`](/home/i4104/repos/babydra-core), làm kim chỉ nam chuẩn hóa cho toàn bộ hệ thống BabyDra Desktop Environment (BabyDraDE).

---

## 1. Triết lý Thiết kế Cốt lõi (Design Philosophy)

Phong cách thiết kế của BabyDra được định hình bởi **Modern Obsidian Acrylic & Precision Glassmorphism**:

1. **Obsidian Deep Matte (Đen mờ sâu, tinh khiết)**:
   * Sử dụng tông màu nền đen than sâu tối giản `rgb(14, 14, 18)` hoặc `rgb(13, 13, 17)` làm nền tảng bề mặt (Surface), mang lại độ tương phản tuyệt đối trên màn hình OLED / IPS.
   * **Tối ưu hiệu năng Wayland**: Khác với các giao diện blur thời gian thực ngốn tài nguyên GPU, BabyDra tối ưu bề mặt với độ đục hoàn chỉnh (`opacity: 1.0` hoặc `rgba(20, 22, 34, 0.98)`), kết hợp với viền mỏng mờ trong suốt (translucent border) tạo cảm giác phân tầng nổi bật mà vẫn giữ tốc độ khung hình ổn định 60fps.

2. **Floating Islands & Capsules (Viên thuốc lơ lửng)**:
   * Toàn bộ các widget thanh điều khiển, trạng thái và Dynamic Island được thiết kế dưới dạng capsule / pill bo cong hoàn mỹ (`border-radius: 9999px`), lơ lửng trên nền desktop có viền mỏng và khoảng đệm thoáng đãng.

3. **No-Arrow Squircle Popovers**:
   * Các cửa sổ bật lên (popover), bảng trạng thái (tooltip) và context menu loại bỏ hoàn toàn các mũi tên tam giác cổ điển (`has_arrow(false)`), sử dụng góc bo mềm `14px - 18px` và drop-shadow sâu lan tỏa `0 16px 48px rgba(0, 0, 0, 0.50)` giúp bề mặt nổi khối tự nhiên.

4. **Tương phản Typography & Tabular Numbers**:
   * Phân cấp văn bản rõ ràng: Tiêu đề in đậm sắc nét, cặp nhãn Key/Value phân tách (Key mờ `50-60%`, Value trắng sáng `95-100%`). Áp dụng font monospace/tabular numbers (`tnum`) cho tất cả các chỉ số động (đồng hồ, pin, CPU, RAM) để ngăn chặn hiện tượng co giật layout.

---

## 2. Hệ thống Token Màu sắc (Color Tokens & Themes)

Hệ thống token được cấu trúc theo 2 tầng: **Tokens cơ bản** (`tokens.json`) và **Lớp phủ chủ đề** (`theme.css` / `dark.css` / `light.css`).

### 2.1. Bảng màu Dark Mode (Chế độ tối mặc định)

| Token | Giá trị CSS | Mô tả ứng dụng |
| :--- | :--- | :--- |
| **`surface`** | `rgb(14, 14, 18)` / `#0e0e12` | Bề mặt chính của popover, capsule, thanh topbar, dock |
| **`surface-card`** | `rgb(24, 25, 31)` / `#18191f` | Khối card bên trong popup, tooltip card |
| **`surface-elevated`**| `rgba(20, 22, 34, 0.98)` | Bề mặt hộp thoại modal, cảnh báo hệ thống |
| **`border`** | `rgba(255, 255, 255, 0.12)` | Viền mỏng thông thường của container và button |
| **`border-subtle`**| `rgba(255, 255, 255, 0.08)` | Đường kẻ phân cách, vạch ngăn cách hàng |
| **`border-highlight`**| `rgba(255, 255, 255, 0.24)` | Viền sáng trên đỉnh hộp thoại modal (`border-top`) |
| **`scrim`** | `rgba(0, 0, 0, 0.65)` | Lớp nền mờ tối khi mở modal / dialog |
| **`text-primary`** | `rgba(255, 255, 255, 0.95)` | Văn bản tiêu đề, giá trị quan trọng, số đo |
| **`text-secondary`**| `rgba(255, 255, 255, 0.60)` | Nhãn thuộc tính (keys), mô tả phụ |
| **`text-muted`** | `rgba(255, 255, 255, 0.40)` | Văn bản không khả dụng, dấu ngoặc trạng thái chờ |
| **`hover-fill`** | `rgba(255, 255, 255, 0.10)` | Nền khi hover chuột vào nút bấm, mục danh sách |
| **`hover-fill-subtle`**| `rgba(255, 255, 255, 0.06)`| Nền khi hover vào ô tìm kiếm hoặc nút nhỏ |

### 2.2. Bảng màu Light Mode (Chế độ sáng)

| Token | Giá trị CSS | Mô tả ứng dụng |
| :--- | :--- | :--- |
| **`surface`** | `#ffffff` | Nền trắng tinh khiết |
| **`border`** | `rgba(0, 0, 0, 0.08)` | Viền mờ xám nhẹ |
| **`text-primary`** | `#111827` / `rgba(0,0,0,0.90)` | Văn bản chính đậm nét |
| **`text-secondary`**| `rgba(0, 0, 0, 0.60)` | Nhãn thuộc tính thứ cấp |
| **`hover-fill`** | `rgba(0, 0, 0, 0.05)` | Hiệu ứng rê chuột |

### 2.3. Bảng màu Accents (Gói chủ đề)

BabyDra hỗ trợ 5 gói màu điểm nhấn chính xác:

| Chủ đề | Accent Dark | Accent Light | Biểu tượng cảm xúc / Phong cách |
| :--- | :--- | :--- | :--- |
| **`babydra-default`** | `#3b82f6` (Blue) | `#3b82f6` | Hiện đại, tiêu chuẩn công nghệ |
| **`babydra-blue`** | `#38bdf8` (Sky) | `#0284c7` | Thanh thoát, bầu trời tươi sáng |
| **`babydra-green`** | `#10b981` (Emerald) | `#047857` | Tự nhiên, êm dịu năng lượng |
| **`babydra-purple`**| `#8b5cf6` (Violet) | `#6d28d9` | Huyền bí, tương lai viễn tưởng |
| **`babydra-rose`** | `#f43f5e` (Rose) | `#be123c` | Ấm áp, cá tính, thời thượng |

### 2.4. Màu Ngữ nghĩa Hệ thống (Semantic Status Colors)

* **Success (Thành công / Tốc độ cao)**: `#22c55e` (Green 500)
* **Warning (Cảnh báo / Tốc độ trung bình)**: `#f59e0b` (Amber 500)
* **Danger (Lỗi / Tốc độ thấp / DND / Nguồn)**: `#ef4444` / `#f43f5e` (Red/Rose 500)
* **Active Indicator (Chấm taskbar, Dot active)**: `#3b82f6`

---

## 3. Hệ thống Typography & Iconography

### 3.1. Font Families

```css
* {
    font-family: 'Quicksand', 'Segoe UI Variable Static Text', 'Segoe UI', 'Inter', system-ui, -apple-system, sans-serif;
    -gtk-icon-style: symbolic;
}

/* Các chỉ số dữ liệu, đồng hồ, đo lường */
.panel-clock,
.status-popover-val,
.sys-monitor-val {
    font-feature-settings: "tnum";
    font-variant-numeric: tabular-nums;
}

/* Khối mã nguồn hoặc terminal */
.code-text,
.log-view {
    font-family: 'JetBrains Mono', 'Fira Code', monospace;
}
```

### 3.2. Cấp bậc Kích cỡ & Độ đậm (Scale & Weights)

| Cấp bậc | Kích cỡ | Font Weight | Line Height | Ứng dụng |
| :--- | :--- | :--- | :--- | :--- |
| **Display / Title Large**| `16px` | `800` (Extra Bold)| `1.2` | Tiêu đề Launcher, Control Center, Hộp thoại chính |
| **Heading Medium** | `14px` | `700` (Bold) | `1.3` | Tiêu đề Card, Header Popover, Tên ứng dụng modal |
| **Body Standard** | `13px` | `500` - `600` | `1.4` | Nội dung văn bản, danh sách cấu hình, ô nhập liệu |
| **Caption / Status** | `12px` | `600` (Semi Bold) | `1.2` | Đồng hồ topbar, nhãn popover, phím tắt |
| **Micro / Subtitle** | `9px` - `10px` | `700` (Bold) | `1.0` | Dấu ngăn cách capsule, nhãn badge nhỏ, nhãn phụ tile |

### 3.3. Quy chuẩn Icon Symbolic

* Mọi icon trong giao diện bắt buộc tuân thủ `-gtk-icon-style: symbolic` để tự động ăn màu theo ngữ cảnh (monochrome).
* **Kích thước pixel chuẩn**:
  * `16px`: Icon danh sách context menu, icon taskbar con, icon status indicator.
  * `14px`: Icon bên trong Dynamic Island notch (`notch-content`).
  * `20px` - `24px`: Quick tile trong Control Center, nút hành động chính.
  * `32px` - `42px`: Huy hiệu squircle trong hộp thoại (`.modern-dialog-badge`).
  * `48px`: Icon tệp tin & thư mục trên desktop grid.

---

## 4. Hệ thống Bo góc & Đổ bóng (Radius & Shadows)

### 4.1. Quy chuẩn Radius (Border Radius Tokens)

```text
pill   : 9999px  →  Capsule thanh topbar, Dynamic Island, status pills, nút hành động chính
lg     : 20px - 24px → Cửa sổ Launcher, Control Center, Calendar, Content Cards, Dialogs
md     : 14px - 18px → Popover tooltips, Context menus, Quick tiles, Card con
sm     : 8px - 12px  → Nút con, ô nhập text (entry), dropdown, huy hiệu icon badge
micro  : 4px - 6px   → Nút bấm thu nhỏ, app icon taskbar, thẻ tag
```

### 4.2. Quy chuẩn Drop Shadows

* **Popover Floating Elevation**:
  ```css
  box-shadow: 0 16px 48px rgba(0, 0, 0, 0.50);
  /* Phiên bản tooltip siêu tối */
  box-shadow: 0 16px 40px rgba(0, 0, 0, 0.65);
  ```
* **Notification Card Elevation**:
  ```css
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.25);
  ```
* **Flat Surfaces**:
  * Toàn bộ thanh topbar, taskbar, island notch trên màn hình desktop giữ `box-shadow: none` để đảm bảo độ phẳng, hòa trộn tuyệt đối vào hình nền.

---

## 5. Quy chuẩn Chi tiết từng Thành phần Giao diện

### 5.1. Topbar & Floating Capsules

* **Kích thước thanh**: Chiều cao `36px`, margin top `8px`, lùi trái/phải `1px`.
* **Cấu trúc 3 phân khu**:
  1. **Left Section**: Arch Logo Button (`20px x 20px`), dấu gạch phân cách capsule `|` (`rgba(255, 255, 255, 0.14)`), tên ứng dụng đang focus (`font-size: 13px; font-weight: 700`), App Menu điều khiển D-Bus.
  2. **Center Section**: Dynamic Island (`.panel-notch`) chuyển đổi linh hoạt.
  3. **Right Section**: Khối Taskbar / System Monitor (`.sys-monitor-box`), Status Indicators (`.status-indicators-box`), Tray (`.panel-tray-box`), Đồng hồ (`.panel-clock-btn`).
* **CSS Capsule mẫu**:
  ```css
  .status-indicators-box,
  .sys-monitor-box,
  .panel-tray-box {
      background-color: rgb(14, 14, 18);
      border: 1px solid rgba(255, 255, 255, 0.12);
      border-radius: 9999px;
      padding: 2px 8px;
      min-height: 26px;
      box-shadow: none;
  }
  ```

### 5.2. Dynamic Island (Notch & Indicators)

* **Notch Container**:
  * Khi ở trạng thái rảnh: Kích thước thu nhỏ tối đa, ẩn mượt mà.
  * Khi kích hoạt media / recording / pin: Bo tròn `border-radius: 9999px`, padding `0 10px`, min-height `26px`.
* **Cặp dấu ngoặc định vị (`.island-bracket`)**:
  * Ký hiệu `(` và `)` nằm ở 2 đầu notch khi có nhiều tính năng đồng thời.
  * Màu sắc: `rgba(255, 255, 255, 0.40)`. Khi hover vào đảo, chuyển sang xanh sáng `#60a5fa`.
  * Font: In hoa đậm nét `font-weight: 800; font-size: 12px`.
* **Visualizer Bars (Sóng âm thanh)**:
  * 4 thanh sóng kích thước `width: 3px`, bo góc `9999px`, nền `#3b82f6` (hoặc accent hiện tại), thay đổi chiều cao sinh động `4px - 14px`.

### 5.3. Popover Tooltips (Audio, Network, Battery, SysMon)

* **Loại bỏ mũi tên**: `has_arrow(false)`.
* **Background & Border**:
  ```css
  popover.status-popover > contents,
  popover.tooltip-popover > contents {
      background-color: rgb(13, 13, 17);
      opacity: 1.0;
      border: 1px solid rgba(255, 255, 255, 0.10);
      border-radius: 18px;
      box-shadow: 0 16px 40px rgba(0, 0, 0, 0.65);
      padding: 14px 18px;
  }
  ```
* **Bố cục Key-Value**:
  * Cột trái (Key): Căn lề trái, màu `rgba(255, 255, 255, 0.50)`, kích thước `12px`, `font-weight: 500`.
  * Cột phải (Value): Căn lề phải, màu `#ffffff`, kích thước `12px`, `font-weight: 700`, sử dụng `tabular-nums`.

### 5.4. Context Menu (Menu chuột phải)

* **Vị trí**: Nổi chính xác tại tọa độ chuột `at_coords(x, y)`.
* **Quy cách**:
  * Chiều rộng tối thiểu: `190px - 200px`.
  * Bo góc: `16px`.
  * Hàng item: Padding `8px 12px`, icon `16px`, khoảng cách nhãn `8px`.
  * Hiệu ứng hover: `background-color: rgba(255, 255, 255, 0.08); border-radius: 8px;`.
  * Đường phân cách: Đường mảnh `margin: 4px 6px; background-color: rgba(255, 255, 255, 0.10); min-height: 1px;`.

### 5.5. Nút gạt Toggle Switch (Cairo Custom Drawing)

* **Kích thước**: `46px` chiều rộng, `24px` chiều cao.
* **Hoạt họa**: Chạy ở tần số 60fps mượt mà bằng phương trình `ease_out_cubic`:
  $$\text{progress}(t) = 1.0 - (1.0 - t)^3$$
* **Màu sắc**:
  * Trạng thái Inactive: Track màu xám đậm `#373c48` (`TRACK_DARK_RGBA`), viền `#292d36`.
  * Trạng thái Active: Track nội suy chuyển sắc sang xanh Accent `#3b82f6` (`ACCENT_RGB`).
  * Thumb (Knob): Tròn trắng `#ffffff`, đổ bóng phía sau `rgba(0, 0, 0, 0.30)`.

### 5.6. Dialog & Modal Hộp thoại

* **Lớp nền Scrim**: Phủ đen mờ `rgba(0, 0, 0, 0.65)`.
* **Hộp thoại chính (`.modern-modal-card`)**:
  * Bo góc: `20px`.
  * Padding: `24px`.
  * Nền: `rgba(20, 22, 34, 0.98)`.
  * Viền: Viền mờ xung quanh `1px solid rgba(255, 255, 255, 0.14)`, viền sáng phản quang đỉnh `border-top: 1px solid rgba(255, 255, 255, 0.24)`.
* **Huy hiệu Squircle Badge (`.modern-dialog-badge`)**:
  * Kích thước: `42px x 42px`, bo góc `12px`.
  * Nền: `rgba(255, 255, 255, 0.08)`, viền `1px solid rgba(255, 255, 255, 0.12)`.
* **Nút bấm hành động (`.btn-primary`)**:
  * Bo tròn hoàn toàn `border-radius: 9999px`, nền `#3b82f6`, màu chữ `#ffffff`, padding `8px 18px`.

---

## 6. Chuyển động & Hoạt họa (Motion & Animations)

1. **Micro-interactions (Tương tác vi mô)**:
   * Tất cả các nút bấm, icon hover, thay đổi màu sắc dùng hiệu ứng:
     ```css
     transition: all 200ms ease;
     ```
2. **Dynamic Island Morphing**:
   * Chuyển động kích thước notch capsule và bung mở popover:
     ```css
     transition: all 400ms cubic-bezier(0.16, 1, 0.3, 1);
     ```
3. **Startup Lag Prevention**:
   * Khi ứng dụng vừa bật, lớp CSS `.startup-animating` được kích hoạt để triệt tiêu toàn bộ transition (`transition: none;`), ngăn chặn tình trạng chớp nháy (flash of unstyled content) khi render khung hình đầu tiên.

---

## 7. Cấu trúc Quản lý Cấu hình và Tệp tin

* **Đường dẫn thư mục**: Toàn bộ cấu hình hệ thống lưu trữ tại thư mục gốc `~/.babydra/`:
  * `~/.babydra/babydra.conf`: Cấu hình TOML chính.
  * `~/.babydra/configs/desktop.json`: Tọa độ icon desktop debounced.
  * `~/.babydra/wallpaper/`: Thư mục chứa hình nền người dùng và vector `default.svg`.
  * `~/.babydra/logs/`: Thư mục log ứng dụng.

