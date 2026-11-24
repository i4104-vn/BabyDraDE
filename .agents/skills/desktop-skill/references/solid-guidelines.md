# Quy Tắc SOLID & Chống Code Thừa (Rust)

---

## 1. SOLID Áp Dụng Cho Rust

### S — Single Responsibility (SRP)
- Mỗi struct/module chỉ làm **một việc** duy nhất.
- Dấu hiệu vi phạm: struct có field không liên quan nhau, function dài hơn 30 dòng.
- **Fix**: Tách thành struct nhỏ hơn hoặc extract hàm helper `fn`.

### O — Open/Closed (OCP)
- Mở rộng bằng cách **thêm impl mới cho trait**, không sửa code cũ.
- Dùng trait objects (`dyn Trait`) hoặc generics thay vì `match` phân nhánh theo loại.

```rust
// ❌ Vi phạm OCP — phải sửa match mỗi khi thêm loại mới
fn render(widget: WidgetKind) {
    match widget {
        WidgetKind::Button => { /* ... */ }
        WidgetKind::Label  => { /* ... */ }
    }
}

// ✅ Đúng — thêm loại mới chỉ cần impl thêm
trait Renderable { fn render(&self); }
fn render(widget: &dyn Renderable) { widget.render(); }
```

### L — Liskov Substitution (LSP)
- Mọi impl của trait phải thực sự thực hiện đúng contract của trait đó.
- Không dùng `unimplemented!()` hay `todo!()` trong production code.

### I — Interface Segregation (ISP)
- Trait phải nhỏ và tập trung. Nếu implementor buộc phải để trống một method của trait → trait đó cần tách.

```rust
// ❌ Trait quá lớn
trait Widget { fn render(&self); fn handle_click(&mut self); fn serialize(&self) -> Vec<u8>; }

// ✅ Tách nhỏ theo trách nhiệm
trait Renderable  { fn render(&self); }
trait Interactive { fn handle_click(&mut self); }
trait Serializable{ fn serialize(&self) -> Vec<u8>; }
```

### D — Dependency Inversion (DIP)
- Module cấp cao phụ thuộc vào **trait (abstraction)**, không phụ thuộc vào concrete struct.
- Inject dependency qua constructor hoặc generic parameter.

```rust
// ❌ Phụ thuộc cứng vào concrete type
struct App { db: PostgresDb }

// ✅ Phụ thuộc vào trait
struct App<S: Storage> { db: S }
```

---

## 2. Chống Code Thừa (DRY — Don't Repeat Yourself)

### Nhận diện code thừa (Code Smell Checklist)
- Cùng một đoạn logic xuất hiện > 2 lần ở các nơi khác nhau.
- Nhiều struct có cùng tập field (candidate để extract shared base/trait).
- Nhiều `match` phân nhánh cùng một enum ở nhiều nơi.

### Hành động khắc phục
| Code Smell | Fix |
| :--- | :--- |
| Logic lặp > 2 lần | Extract thành `fn` hoặc method trên trait |
| Nhiều struct cùng field | Extract trait hoặc dùng composition (newtype) |
| Nhiều `match` cùng enum | Chuyển thành method của enum |
| Cùng pattern khởi tạo | Extract `fn new_xxx() -> T` builder/constructor |

---

## 3. Quy Tắc Viết Hàm & Struct Sạch

```
Hàm:  ≤ 30 dòng  |  ≤ 3 tham số  |  1 trách nhiệm
Struct: ≤ 7 fields  |  nếu nhiều hơn → tách thành sub-struct
File:  ≤ 200 dòng  |  nếu vượt → tách sub-module
```

- **Ưu tiên `Option` / `Result`** thay vì sentinel value (`-1`, `""`, `null`).
- **Tránh boolean trap**: Thay `fn set(enabled: bool)` bằng enum `fn set(state: EnableState)`.
- **Tránh magic number**: Đặt tên constant `const BORDER_RADIUS: f64 = 14.0;` thay vì hardcode.

---

## 4. Quy Chuẩn Xử Lý Lỗi

```rust
// Trong libs/ — dùng thiserror, explicit error types
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Config load failed: {0}")]
    Config(String),
    #[error("D-Bus unavailable")]
    DBus,
}

// KHÔNG dùng unwrap()/expect() trong runtime path
// KHÔNG dùng anyhow trong libs/ (chỉ dùng ở binary crate nếu cần)
// Dùng tracing cho log context
tracing::error!(%err, "Fatal in event loop");
```

