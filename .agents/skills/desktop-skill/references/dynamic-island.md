# Dynamic Island — Topbar Crate

> Applies to `crates/features/topbar/src/center/` only.

## Golden Rules

1. **One API:** Every feature implements `IslandFeature::build(&mut self, ctx: &IslandCtx)`. No `widget()`+`init()`+`attach()` trio.
2. **One context:** `IslandCtx` is the only object a feature receives (view handle, capsule, tick, popover, keyboard, IPC).
3. **One registration:** `center/registry.rs` calls `engine.register_feature(...)`. Nothing else.
4. **One IPC:** `center/services/ipc.rs` owns D-Bus (`org.babydra.Island`). Features use `ctx.on_trigger(...)`.
5. **One poller:** `center/services/poller.rs::spawn_poller`. Feature's `service.rs` only exposes `read()`.
6. **One widget kit:** `center/ui/notch.rs` + `center/ui/popover.rs`. Features compose, never reimplement.
7. **Max 200 lines per file.**

## Directory Layout

```text
center/
├── mod.rs                  # Public exports only
├── registry.rs             # register_all(&mut IslandEngine)
├── engine/                 # Framework core (state machine, no GTK feature code)
│   ├── mod.rs              # IslandEngine: register, tick, dispatch
│   ├── builder.rs          # Widget-tree construction
│   ├── feature.rs          # IslandFeature trait + capsule size constants
│   ├── context.rs          # IslandCtx (handle + shared services)
│   ├── handle.rs           # ViewState + IslandViewHandle
│   ├── record.rs           # Internal ViewRecord
│   └── arbitration.rs      # refresh_requests + select_winner
├── services/
│   ├── mod.rs
│   ├── ipc.rs              # GIO D-Bus + trigger topic registry
│   └── poller.rs           # Generic worker-thread poller → GTK main context
├── ui/
│   ├── mod.rs
│   ├── capsule.rs          # IslandCapsule: engine owner, 150ms tick
│   ├── notch.rs            # NotchWidget: icon | title | value layout
│   ├── popover/
│   │   ├── mod.rs          # IslandPopover: new_slide / new_modal
│   │   ├── registry.rs     # Open-popover tracking, Escape, outside-click
│   │   ├── lifecycle.rs    # map/unmap: CSS, focus, slide-in
│   │   └── animation.rs    # toggle + cancelable slide-out
│   └── transition.rs       # Expand/collapse capsule animations
└── features/               # One folder per feature
    └── <name>/
        ├── mod.rs          # Struct + IslandFeature impl (≤ 200 lines)
        ├── service.rs      # One read() or data helpers (no GTK)
        ├── controller/     # Optional: keyboard.rs / pointer.rs
        └── ui/             # Optional: layout + render helpers
```

## Arbitration Order

1. Open popover always wins (never hijacked).
2. Most recent `override_show_for` wins.
3. Highest `priority()`, tie-broken by most recent `show()`.
4. Deadlines purged each tick.
5. `hover_keep()` keeps view alive only while pointer is over capsule.

## Adding a Feature

### 1. Create module

```rust
// center/features/<name>/mod.rs
pub mod service;

use crate::center::{IslandCtx, IslandFeature, NotchWidget, CAPSULE_HEIGHT, CAPSULE_WIDTH};
use crate::center::poller::spawn_poller;

pub struct MyFeature { widgets: NotchWidget }

impl MyFeature {
    pub fn new() -> Self {
        Self { widgets: NotchWidget::with_value("icon", "Label", "") }
    }
}

impl IslandFeature for MyFeature {
    fn id(&self) -> &'static str { "my_feature" }
    fn priority(&self) -> u32 { 90 }
    fn size(&self) -> (i32, i32) { (CAPSULE_WIDTH, CAPSULE_HEIGHT) }
    fn is_alive(&self) -> bool { false }

    fn build(&mut self, ctx: &IslandCtx) -> gtk4::Widget {
        let widgets = self.widgets.clone();
        let ctx = ctx.clone();
        spawn_poller("island-my", Duration::from_secs(1), service::read, move |val| {
            widgets.update("icon", &val.to_string());
            ctx.override_show_for(Duration::from_secs(2));
            ctx.tick();
        });
        self.widgets.container.clone().upcast()
    }
}
```

### 2. Register

`features/mod.rs`: `pub mod my_feature;`
`center/registry.rs`: `engine.register_feature(MyFeature::new());`

### 3. Optional: popover + keyboard

```rust
let popover = MyPopover::new(&ctx.capsule()); // IslandPopover::new_modal
ctx.attach_keyboard(&popover.popover, &popover.popover_box, move || {
    keyboard_controller(popover_rc.clone(), ctx_for_keys.clone())
});
// Override is_popover_open() → self.popover.borrow().as_ref().is_some_and(|p| p.is_visible())
```

### 4. Optional: IPC trigger

```rust
ctx.on_trigger(|| { ctx.override_show_for(Duration::from_secs(5)); ctx.tick(); });
```

Topic = feature's `id()`. Never create another D-Bus server.

### 5. Checklist

- [ ] `build(ctx)` implemented; no old `widget/init/attach` hooks
- [ ] No handle stored in struct; all requests via `ctx`
- [ ] Background work uses `spawn_poller` + `service::read()`
- [ ] Popovers use `IslandPopover::new_slide`/`new_modal`
- [ ] Keyboard uses `ctx.attach_keyboard`
- [ ] IPC uses `ctx.on_trigger` only
- [ ] One line in `registry.rs`; no other framework file changed
- [ ] Every file ≤ 200 lines; `cargo check -p babydra-topbar` clean

## Adaptations from babydra-island

| Upstream | BabyDraDE topbar |
|:--|:--|
| `zbus` D-Bus | `gio` low-level registration |
| `babydra_core` services | `libs/services` equivalents |
| `babydra_ui_kit` | `libs/ui-kit` |
| `island::default_island()` | Weak engine ref in `IslandCtx` |
