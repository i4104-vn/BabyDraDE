# Protocol bus — kiến trúc và cơ chế hoạt động

Tài liệu này mô tả toàn bộ đường truyền thông điệp giữa các feature app và
compositor, từ định dạng trên dây đến cách router xử lý từng loại frame. Đây là
nguồn sự thật duy nhất cho protocol sau khi crate `protocol` độc lập bị bãi bỏ.

## 1. Mục tiêu và nguyên tắc

- **Một router, mọi peer đều bình đẳng.** WM không phải "server đặc biệt" mà là
  một peer tên `wm` cắm vào cùng router. Feature có thể gọi feature khác.
- **Không file, không polling theo frame.** Vận chuyển là abstract Unix socket
  của Linux; có dữ liệu là calloop đánh thức ngay. Không có thư mục queue, không
  có `babydra-active-window.json`, không có manifest shortcut trên đĩa.
- **Tất cả trạng thái nằm trong RAM.** Outbox, pending request, retained topic,
  offline queue được xoá ngay sau khi hoàn tất, timeout hoặc peer ngắt kết nối.
- **Contract tách theo quyền sở hữu.** Wire types dùng chung nằm ở `common`;
  client của feature nằm ở `services`; router và endpoint của compositor nằm
  trong `window_manager`. Không crate nào phải phụ thuộc ngược vào compositor.

## 2. Sở hữu mã nguồn

| Crate / module | Đường dẫn | Trách nhiệm |
|:--|:--|:--|
| `common::protocol` | `libs/common/src/protocol/` | `Envelope`, `Message`, `Target`, `Kind`, `BusError`, `Control`, `Request`/`Response`/`Event`, codec, socket client, `Writer` |
| `services::protocol` | `libs/services/src/protocol/` | `Bus` — client dùng bởi feature app |
| `window_manager::protocol` | `crates/window-manager/src/protocol/` | `Router`, `Connection`, `bind()`, `BusSender` |

`libs/services/src/system/shortcuts/` và `libs/services/src/system/window/`
là người dùng của `Bus`; chúng không tự định nghĩa giao thức.

### 2.1 Cách chia file

Mỗi module được tách theo **một trách nhiệm duy nhất**, tuân thủ quy tắc của
`desktop-skill`: mọi `use` nằm ở đầu file, `mod.rs` chỉ chứa khai báo/re-export,
không inline `mod` chứa import, không file `foo.rs` đứng cạnh thư mục `foo/`.
Đây là lý do protocol không còn nằm gộp trong vài file dài hàng trăm dòng.

```text
libs/common/src/protocol/
├── mod.rs          # khai báo + re-export flat; không có logic
├── envelope.rs     # Envelope, Message, Incoming, Target, Kind, Control, BusError
├── models.rs       # Request / Response / Event / WindowAction / ActiveWindowInfo
├── codec.rs        # encode/decode + LineBuffer + CodecError (1 dòng JSON, 1 MiB)
├── socket.rs       # abstract_addr, connect, verify_peer (SO_PEERCRED)
└── writer.rs       # Writer: id seed theo pid+time, ghi frame, shutdown

libs/services/src/protocol/
├── mod.rs          # pub use Bus + re-export wire types cho consumer
├── state.rs        # BusState, Inbox, ReaderPending, frame routing, reader loop, handshake
├── bus.rs          # vòng đời Bus: connect, name, emit, prune, shutdown
├── bus_io.rs       # API công khai: send/request/respond/publish/recv/...
└── tests.rs        # test state transition, parse lỗi, correlation, socket round-trip

crates/window-manager/src/protocol/
├── mod.rs          # khai báo + re-export (common::protocol::* + Router/BusSender)
├── router.rs       # state machine thuần: routing, pending, retained, offline, tick
├── connection.rs   # ReadResult + Connection (đọc/ghi nonblocking có buffer)
├── socket.rs       # chỉ còn bind(): UnixListener abstract, nonblocking
├── sender.rs       # BusSender dùng Writer của common cho peer `wm`
└── tests.rs        # test router, connection, BusSender

libs/services/src/system/shortcuts/     # consumer, không phải protocol core
├── mod.rs          # re-export ShortcutClient / ShortcutError / read_all
├── error.rs        # ShortcutError
├── transport.rs    # connect bus, chờ response, decode BusError/Response
├── client.rs       # ShortcutClient API
├── snapshot.rs     # read_all, group manifest theo owner
└── tests.rs        # rule thuần (owner, NotFound, filter, group)
```

Trước khi tách: `services/protocol.rs` 793 dòng và
`services/system/shortcuts.rs` 653 dòng. Sau khi tách, file lớn nhất của services
là `transport.rs` 278 dòng; router của WM là 816 dòng nhưng chỉ chứa logic state
machine liên quan chặt chẽ với nhau (skill cho phép file dài khi related code
nằm cùng nhau), còn test đã đưa hết sang `tests.rs`.

## 3. Topology

```text
                  abstract socket  \0babydra-bus-<uid>
                                 │
    ┌────────────┬───────────────┼───────────────┬────────────┐
    │            │               │               │            │
┌───▼───┐   ┌────▼────┐     ┌────▼────┐     ┌────▼────┐  ┌───▼───┐
│topbar │   │services │     │greeter  │     │desktop  │  │  wm   │
│ Bus   │   │ Bus     │     │ Bus     │     │ Bus     │  │Sender │
└───┬───┘   └────┬────┘     └────┬────┘     └────┬────┘  └───┬───┘
    └────────────┴───────────────┼───────────────┴───────────┘
                                 ▼
                    ┌────────────────────────┐
                    │ Router (trong WM)      │
                    │ peers · topics         │
                    │ pending · retained     │
                    │ offline · subscriptions│
                    └────────────────────────┘
```

WM tạo router ngay lúc khởi tạo state, trước khi autostart feature. Socket đặt
trong **abstract namespace** của Linux (`SocketAddrExt::from_abstract_name`),
nên không có inode trên filesystem và tự biến mất khi process kết thúc. Tên có
UID để tránh xung đột giữa các phiên người dùng.

## 4. Định dạng trên dây

Mỗi message là **một dòng JSON kết thúc bằng `\n`**, tối đa 1 MiB. Người ta có
thể `cat` queue cũ để debug; giờ vẫn giữ framing này để tái dùng codec và dễ
soi luồng.

```json
{"version":2,"id":7,"source":"org.babydrade.topbar","target":{"peer":"wm"},"kind":"request","retained":false,"ttl_ms":null,"payload":{...}}
```

### 4.1 `Envelope`

| Trường | Kiểu | Ý nghĩa |
|:--|:--|:--|
| `version` | `u16` | Phiên bản wire, hiện tại **2**. Khác版本 bị từ chối bằng `Kind::Error`. |
| `id` | `u64` | Correlation id do bên gửi cấp; không trùng giữa các process. |
| `source` | `String` | Router **tự điền** tên peer khi nhận; client gửi rỗng nên không giả mạo được. |
| `target` | `Target` | Đích cần đến (xem bên dưới). |
| `kind` | `Kind` | `request`, `response`, `event`, `error`. |
| `retained` | `bool` | `true` khi frame là bản replay từ cache retained. |
| `ttl_ms` | `Option<u64>` | Hạn sống cho retained topic / offline queue. |
| `payload` | `Value` | Thân message; giải mã bằng `Message::decode::<T>()`. |

### 4.2 `Target`

| Variant | Hành vi |
|:--|:--|
| `Peer(String)` | Giao cho đúng peer đã đăng ký tên đó. Request bắt buộc dùng dạng này. |
| `Topic(String)` | Giao cho mọi subscriber của topic, trừ chính người gửi. |
| `Broadcast` | Giao cho mọi peer sẵn sàng, trừ người gửi. |
| `Router` | Dành riêng cho control frame (`Hello`, `Subscribe`, `Unsubscribe`). |

### 4.3 `Kind`

- `Request` — cần một `Response` có cùng `id`.
- `Response` — router **bỏ qua `target`**, tra pending theo `(peer gửi, id)`.
- `Event` — một chiều, có thể retained.
- `Error` — do router hoặc peer báo lỗi, body là `BusError`.

`Message` là dạng đã giải mã đưa cho ứng dụng: `{id, source, kind, retained,
topic, body}`. `topic` chỉ có giá trị với `Target::Topic`.

### 4.4 Contract của service `wm`

Nằm trong `common::protocol` và không dính transport:

| Request | Response |
|:--|:--|
| `WindowAction { action }` | `Ok` |
| `CloseTargeted { target }` | `Ok` / `Error(NotFound)` |
| `ShortcutRegister { owner, binding }` | `Ok` / `Error(Failed)` |
| `ShortcutUnregister { owner, id }` | `Ok` / `Error(NotFound)` |
| `ShortcutList` | `Shortcuts { entries }` |
| `RunCommand { command }` | `Ok` |
| `SystemAction { target }` | `Ok` |
| `Ping` | `Ok` |

`Event` hiện dùng: `ActiveWindowChanged`, `CompositorReady`, `Shutdown`,
`ShortcutsChanged`. Focus cửa sổ phát dạng **retained** trên topic
`window.active` với TTL 60 giây.

## 5. Router — cơ chế từng bước

### 5.1 Kết nối và đăng ký

1. Feature gọi `Bus::connect(name)` → mở abstract socket, gửi `Hello { name }`
   tới `Target::Router`.
2. Router cấp `PeerId`, lưu `name → PeerId`. Trùng tên → `BusError::NameTaken`
   rồi đóng.
3. Router trả `Welcome { name, peers }`; client lưu danh sách peer và khởi động
   reader thread.
4. Router phát `Control::PeerJoined` tới topic `bus/peers`; khi peer ngắt thì
   phát `PeerLeft`. Client tự cập nhật cache `bus.peers()`.

### 5.2 Giao một request

```text
A: Bus::request("wm", body)  id=7
   └─ Envelope{kind:Request, target:Peer("wm"), id:7}
        ▼
Router: source = "A"; pending[(wm_id,7)] = (A, now)
        └─ outbox(wm)
             ▼
WM: bridge::dispatch(message, state)
    ├─ msg.decode::<Request>()
    ├─ handle(id, request, state) → Response
    └─ BusSender::respond(&message, &response)
             ▼
Router: pending[(wm_id,7)] → xoá; outbox(A)
             ▼
A: Incoming::Response(Message{ id:7, source:"wm" })
```

Điểm mấu chốt: **id do mỗi process tự cấp nên router phải đảm bảo không trùng**.
`Writer` khởi tạo id bằng hàm `seed_id()` (`pid × hash + thời gian + sequence`)
trước khi tăng dần, nên hai feature khởi động cùng lúc không bao giờ trùng id.

### 5.3 Response và lỗi

- Response tới: router tra `pending[(sender, id)]`; có thì xoá entry và đẩy về
  origin; không có thì drop (response trễ / giả mạo).
- Request tới peer không tồn tại: trả `Kind::Error` / `PeerUnavailable` ngay;
  request **không bao giờ** được xếp hàng chờ peer offline.
- Sai version hoặc control frame lạ: `InvalidPayload`; sai target cho request:
  `InvalidTarget`.
- Quá 10 giây chưa có response: timer `tick()` của router gửi `Timeout` cho
  origin và xoá pending. Client cũng có timeout 10 giây cục bộ qua
  `Incoming::Timeout`.

### 5.4 Event, retained và offline

```text
WM: publish_retained("window.active", Event::ActiveWindowChanged{info}, 60s)
Router:
  ├─ gửi cho subscriber hiện tại
  └─ retained["window.active"] = (envelope, now + 60s)

Feature mới: subscribe("window.active")
  └─ router gửi ngay bản retained với retained = true
```

- `tick()` xoá retained/offline hết hạn.
- Event gửi tới peer chưa online được giữ tối đa 32 message, TTL 30 giây; khi
  peer `Hello` thành công, queue được flush và xoá.
- Presence và retained giúp feature khởi động muộn vẫn có trạng thái hiện tại
  mà không đọc file.

### 5.5 Vòng đời peer

`remove(peer)` xoá tên, subscription, mọi pending liên quan đến peer đó. Các
request đang chờ peer bị trả `PeerUnavailable`; các request do peer đó khởi tạo
bị bỏ. `PeerLeft` được phát tới subscriber `bus/peers`.

## 6. Tích hợp calloop trong WM

`crates/window-manager/src/app/bus.rs` làm "adapter":

- `Generic<UnixListener>` chế độ `Mode::Level`, `Interest::READ`: accept tới
  `WouldBlock`, kiểm tra `SO_PEERCRED`, rồi tạo source mới.
- Mỗi peer (kể cả `wm`) là `Generic<..., Interest::BOTH, Mode::Edge>`. Edge
  triggered bắt buộc phải đọc/ghi tới `WouldBlock` trong mỗi callback.
- `flush_all()` lấy `Router::take_outbox(id)` rồi `Connection::queue` + `flush`;
  phần gửi tới peer `wm` được **dispatch trực tiếp** vào `bridge::dispatch()`
  thay vì ghi socket. Response do `BusSender` ghi lại vào nửa socketpair kia.
- `Timer::immediate()` lặp mỗi giây gọi `Router::tick(now)` để dọn timeout.

Vì protocol là nguồn sự kiện của calloop, nó chạy ngay cả khi backend udev đang
tạm dừng render khi chuyển VT. `frame_pacing::begin_frame()` **không** gọi
protocol.

### Bảo mật

`common::protocol::verify_peer()` đọc `SO_PEERCRED` bằng `libc::getsockopt` và
so uid với `geteuid()`. Kết nối từ uid khác bị từ chối. (`peer_cred()` của std
vẫn chưa ổn định nên không dùng.)

## 7. Client `Bus` phía feature

| API | Mục đích |
|:--|:--|
| `connect(name)` | Hello/Welcome (5s), khởi động reader thread |
| `send(target, body)` | Event một chiều |
| `request(target, body)` | Request + ghi pending, trả `MessageId` |
| `respond(&message, body)` | Trả lời request, dùng lại `id`/`source` |
| `publish` / `publish_retained` | Pub/sub, retained có TTL |
| `subscribe` / `unsubscribe` | Control frame tới router |
| `try_recv` / `recv_timeout` / `drain` | Lấy `Incoming` từ inbox RAM |
| `peers` | Danh sách peer theo `bus/peers` |
| `shutdown` | Đóng socket, join reader thread |

Bên trong: một `Writer` (`Mutex<UnixStream>`) cho mọi lời gửi, một reader thread
đọc dòng JSON vào `VecDeque<Incoming>` + `Condvar`. Các message có `id` không
khớp được giữ lại trong `deferred`, không bao giờ bị mất. UI chỉ cần gọi
`drain()` định kỳ (topbar dùng timer 50 ms).

## 8. Ví dụ

### Feature gọi WM

```rust
use services::protocol::Bus;
use common::protocol::{Request, WindowAction};

let bus = Bus::connect("org.babydrade.topbar")?;
let id = bus.request("wm", &Request::WindowAction {
    action: WindowAction::Minimize,
})?;
```

### Feature gọi feature khác

```rust
bus.request("org.babydrade.services", &MyPayload { .. })?;
bus.publish("window.active", &snapshot)?;
```

### WM phát trạng thái

```rust
self.bus.publish_retained(
    "window.active",
    &Event::ActiveWindowChanged { info },
    Duration::from_secs(60),
);
```

### Thêm capability mới

1. Thêm variant vào `Request`/`Response`/`Event` trong `common::protocol`.
2. Thêm nhánh xử lý trong `crate::app::bridge::handle()` (hoặc helper riêng).
3. Feature gọi hàm tiện ích trên `Bus`. Không sửa router, codec hay transport.

## 9. Shortcut đi qua bus

`ShortcutClient` (`libs/services/src/system/shortcuts/`) không còn ghi manifest
vào `$XDG_RUNTIME_DIR/babydra-shortcuts`:

- `register` / `bind` → `Request::ShortcutRegister`
- `unregister` → `Request::ShortcutUnregister`
- `bindings` / `read_all` → `Request::ShortcutList`
- `clear` → list rồi unregister từng binding

Compositor nhận trực tiếp tại `app/bridge.rs`; watcher file 300 ms đã bị xoá.
Mỗi client mở một bus connection riêng (`shortcuts.<pid>.<counter>`) và chờ
response có cùng `id`, tối đa 2 giây.

## 10. Giới hạn hiện tại

- Router chạy trong tiến trình WM; nếu WM chết thì mọi bus connection mất.
- Một abstract socket cho mỗi uid, không có nhiều instance compositor cùng uid.
- Không có persistence: retained/offline chỉ sống trong RAM theo TTL.
- `bus/peers` là constant tên topic được định nghĩa ở cả router và client; nếu
  đổi phải đổi cùng lúc.
- Abstract socket bị sandbox kiểu Flatpak chặn; nếu cần, thay bằng filesystem
  socket trong runtime directory có mode 0600 + `verify_peer`.

## 11. Kiểm tra

```bash
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Các test protocol nằm ở `crates/window-manager/src/protocol/tests.rs` (router,
connection, BusSender) và `libs/services/src/protocol/tests.rs` (client, socket
round-trip). Chúng kiểm tra chuyển trạng thái, parse lỗi và correlation — không
test getter hay tạo widget.
