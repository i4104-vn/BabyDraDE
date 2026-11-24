---
name: desktop-skill
description: >-
  Kỹ năng phát triển desktop components cho BabyDraDE bằng Rust/GTK4.
  Kích hoạt khi người dùng yêu cầu: tạo mới component, refactor code, tổ chức cấu trúc module,
  tối ưu codebase, dọn dẹp dead code, hoặc áp dụng nguyên tắc SOLID/Clean Architecture.
---

# Desktop Development Skill — BabyDraDE

Skill này đảm bảo mọi thao tác viết code Rust trong dự án đều tuân thủ nguyên tắc **phân tầng sạch, SOLID, không trùng lặp, dễ mở rộng** và **code được dọn dẹp sau mỗi phiên làm việc**.

---

## Tài Liệu Tham Khảo

- **Kiến trúc & cấu trúc phân tầng**: [architecture.md](./references/architecture.md)
- **Quy tắc SOLID & chống code thừa**: [solid-guidelines.md](./references/solid-guidelines.md)
- **Quy tắc thiết kế component**: [component-design.md](./references/component-design.md)

---

## Quy Trình Mỗi Lần Viết Code

### 1 — Trước khi viết code

1. **Xác định tầng**: Xác định rõ file mới thuộc tầng nào (xem [architecture.md](./references/architecture.md)).
2. **Kiểm tra trùng lặp**: Tìm kiếm logic tương tự đã tồn tại trước khi tạo hàm/struct mới. Nếu có, tái sử dụng hoặc extract thành trait chung.
3. **Thiết kế interface trước**: Viết trait/signature của public API trước, sau đó mới viết implementation.

### 2 — Trong khi viết code

- Mỗi struct/function phải có **một trách nhiệm duy nhất** (SRP).
- Mỗi module **không được vượt quá 200 dòng**; nếu vượt → split thành sub-module.
- Logic tái sử dụng **phải được đặt vào trait** hoặc hàm helper trong `libs/`.
- Không clone/copy logic từ nơi khác; luôn extract thành abstraction dùng chung.
- Áp dụng quy chuẩn từ [solid-guidelines.md](./references/solid-guidelines.md).

### 3 — Dọn dẹp bắt buộc sau mỗi phiên

Sau khi hoàn tất tác vụ, AI **bắt buộc** thực hiện các bước sau:

```powershell
# 1. Kiểm tra warnings và lỗi
cargo check 2>&1

# 2. Phân tích và loại bỏ dead code, unused imports
cargo clippy -- -W clippy::all -W clippy::pedantic 2>&1
```

Sau khi chạy, AI phải:

- **Xóa** mọi `use` import không dùng đến.
- **Xóa** mọi hàm/biến/constant khai báo nhưng không dùng (`#[allow(dead_code)]` chỉ là tạm thời).
- **Flatten** các wrapper không cần thiết (nếu một struct chỉ bọc một field, cân nhắc bỏ wrapper).
- **Merge** các function/closure trùng lặp vào một chỗ chung.
- **Chuyển toàn bộ** các use lên đầu file, ko chứa trong code.
