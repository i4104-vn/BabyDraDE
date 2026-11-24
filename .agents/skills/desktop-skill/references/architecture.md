# Kiến Trúc & Phân Tầng (Clean Architecture — Rust)

Tài liệu này định nghĩa sơ đồ phân tầng cứng (hard layers) cho mọi crate trong dự án.

---

## 1. Quy Tắc Phân Tầng (Layer Rules)

```text
┌─────────────────────────────────────────────────────────┐
│  composition/     Khởi tạo app, wiring, main.rs         │  ← Có thể import TẤT CẢ
├─────────────────────────────────────────────────────────┤
│  infrastructure/  File I/O, D-Bus, Wayland backends      │  ← Import: application, domain
├─────────────────────────────────────────────────────────┤
│  adapters/        GTK4 widgets, event translators        │  ← Import: application, domain
├─────────────────────────────────────────────────────────┤
│  application/     Use cases, state machines, commands    │  ← Import: domain ONLY
├─────────────────────────────────────────────────────────┤
│  domain/          Entities, value objects, pure logic    │  ← Import: NOTHING (stdlib only)
└─────────────────────────────────────────────────────────┘
```

**Quy tắc sắt đá (Hard rules)**:
- Tầng trong **không bao giờ** import tầng ngoài.
- `domain/` không được chứa code liên quan đến GTK4, Smithay, hay I/O.
- `adapters/` không được gọi trực tiếp vào `infrastructure/`.

---

## 2. Cấu Trúc File Tiêu Chuẩn

```text
src/
├── domain/
│   ├── mod.rs             # re-export công khai
│   ├── entities.rs        # Struct thuần dữ liệu
│   └── value_objects.rs   # Newtypes, enums, invariants
├── application/
│   ├── mod.rs
│   ├── use_cases.rs       # Logic nghiệp vụ, state transitions
│   └── ports.rs           # Trait interface (dependency inversion)
├── adapters/
│   ├── mod.rs
│   ├── ui/                # GTK4 widgets
│   └── input_handler.rs   # Chuyển đổi input events → domain types
├── infrastructure/
│   ├── mod.rs
│   └── config_loader.rs   # File I/O, D-Bus, backends
└── composition/
    └── mod.rs             # Wiring, DI, bootstrap
```

---

## 3. Quy Tắc Tổ Chức Module

- Mỗi module chỉ export qua `mod.rs` — không export trực tiếp từ file con ra ngoài crate.
- Giới hạn tối đa **200 dòng/file**; vượt quá → tách sub-module.
- Mỗi thư mục phải có `mod.rs` hoặc `lib.rs`; không để file lơ lửng ngoài module tree.
- Sử dụng `pub(crate)` thay vì `pub` cho các item chỉ dùng nội bộ crate.

