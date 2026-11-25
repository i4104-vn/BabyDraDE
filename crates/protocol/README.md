# `protocol` — bridge giữa features và window-manager

Crate này là **hợp đồng có kiểu** giữa các app feature (`crates/features/*`)
và compositor (`crates/window-manager`). Hai bên không import lẫn nhau; cả hai
chỉ link crate này.

## Luồng dữ liệu

```text
feature app                              window-manager
  FeatureBridge::send_request(Request) ──▶ ProtocolServer::drain_requests()
  FeatureBridge::poll_events()        ◀── ProtocolServer::publish(Event)
  FeatureBridge::drain_responses()    ◀── ProtocolServer::respond(Response)
```

Mọi message là một `Envelope` có `version`, `id` và `payload` (`Request`,
`Response` hoặc `Event`). Codec mã hóa mỗi envelope thành **một dòng JSON**,
giúp debug bằng `cat` và tránh đọc nửa message.

## Cấu trúc thư mục

```text
src/
├── lib.rs              facade + re-export công khai
├── models/             dữ liệu trên dây (không logic)
│   ├── envelope.rs     phiên bản, id, Payload
│   ├── request.rs      Request: window, shortcut, command, system, ping
│   ├── response.rs     Response thành công / lỗi
│   ├── event.rs        Event compositor phát ra
│   └── types.rs        WindowAction, ActiveWindowInfo
├── codec/              serialize / deserialize + kiểm tra kích thước, phiên bản
├── transport/          trait Transport + FileTransport (queue atomic theo file)
│   └── paths.rs        bố cục thư mục runtime
├── client/             FeatureBridge dùng bởi feature app
└── server/             ProtocolServer, Dispatcher, handler theo capability
    └── handlers/       window.rs, shortcut.rs, NullHandler, ChainHandler
```

## Thêm một capability mới

1. Thêm variant vào `models/request.rs` (và response/event nếu cần).
2. Thêm một file trong `server/handlers/` implement `RequestHandler`.
3. Ghép handler vào `ChainHandler` trong `app/bridge.rs` của window-manager.
4. Feature gọi hàm tiện ích mới trên `FeatureBridge`.

Không sửa codec hay transport — chúng làm việc với `Envelope` chung.

## Mở rộng

`FileTransport` có thể thay bằng transport Unix socket trong tương lai mà không
đổi payload hay client/server API. Mọi action hiện đều đi qua bridge; không còn
đường dẫn file action cũ.
