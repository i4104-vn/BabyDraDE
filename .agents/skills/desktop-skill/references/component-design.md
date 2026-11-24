# Quy Tắc Thiết Kế Component (GTK4 / Rust)

---

## 1. Anatomy Tiêu Chuẩn Của Một Component

Mỗi UI component **phải** được tổ chức theo 4 phần sau, theo đúng thứ tự:

```rust
// ─── 1. Config / Props ───────────────────────────────────────────
// Chứa toàn bộ tham số đầu vào — không có logic, chỉ có data.
#[derive(Debug, Clone)]
pub struct ButtonConfig {
    pub label: String,
    pub icon:  Option<String>,
    pub style: ButtonStyle,  // enum, không dùng raw string
}

// ─── 2. Widget Struct (State + Root Widget) ──────────────────────
// Chỉ chứa root container và các sub-widget cần cập nhật runtime.
// Không chứa logic nghiệp vụ.
pub struct PrimaryButton {
    root:  gtk4::Button,
    label: gtk4::Label,
}

// ─── 3. Constructor (new) ─────────────────────────────────────────
// Khởi tạo widget tree + áp dụng CSS class. KHÔNG gắn signal ở đây.
impl PrimaryButton {
    pub fn new(cfg: &ButtonConfig) -> Self {
        let label = gtk4::Label::new(Some(&cfg.label));
        let root  = gtk4::Button::new();
        root.set_child(Some(&label));
        root.add_css_class("primary-button");
        Self { root, label }
    }

    // ─── 4. Public API (cập nhật trạng thái) ─────────────────────
    pub fn set_label(&self, text: &str) { self.label.set_text(text); }
    pub fn widget(&self) -> &gtk4::Button { &self.root }
}
```

> **Quy tắc**: Gắn signal (`.connect_clicked`, ...) **bên ngoài** struct, trong tầng `adapters/` hoặc `composition/`, không nhúng vào trong `new()`.

---

## 2. Design Tokens Cứng (Không Được Hardcode Trực Tiếp)

Toàn bộ giá trị visual phải đặt qua **CSS class** hoặc **named constant**, không hardcode inline:

```rust
// ❌ Sai
widget.set_margin_start(12);
widget.set_opacity(0.95);

// ✅ Đúng — đặt trong file .css của dự án, áp dụng qua class
widget.add_css_class("surface-primary");
widget.add_css_class("text-primary");
```

Bảng token chuẩn (CSS class):

| Class | Giá trị CSS | Dùng cho |
| :--- | :--- | :--- |
| `surface-primary` | `background: rgb(14,14,18)` | Nền chính |
| `surface-card` | `background: rgb(24,25,31)` | Card bên trong |
| `border-std` | `border: 1px solid rgba(255,255,255,0.12)` | Viền mặc định |
| `radius-capsule` | `border-radius: 9999px` | Capsule / pill shape |
| `radius-card` | `border-radius: 14px` | Card, popover |
| `text-primary` | `color: rgba(255,255,255,0.95)` | Tiêu đề, giá trị |
| `text-secondary` | `color: rgba(255,255,255,0.60)` | Nhãn, mô tả |
| `text-muted` | `color: rgba(255,255,255,0.40)` | Disabled / placeholder |

---

## 3. Các Trạng Thái Bắt Buộc

Mọi component có tương tác **phải** xử lý đủ 4 trạng thái:

| Trạng thái | CSS class / GTK state | Chú thích |
| :--- | :--- | :--- |
| Default | *(không class extra)* | Hiển thị bình thường |
| Hover | `widget.add_css_class("hover")` hoặc CSS `:hover` | Transition 150ms ease-out |
| Active / Pressed | CSS `:active` | Scale down nhẹ hoặc highlight border |
| Disabled | `widget.set_sensitive(false)` | opacity 0.4, không nhận event |

---

## 4. Quy Tắc Animation

- Dùng `libs/animation` (spring physics) thay vì CSS `transition` cho chuyển động phức tạp.
- CSS `transition` chỉ dùng cho micro-interaction đơn giản (opacity, color).
- Thời lượng khuyến nghị:
  - Hover / Toggle: `120ms – 180ms`
  - Popover mở/đóng: `200ms – 280ms`
  - Workspace switch: `300ms – 350ms`
