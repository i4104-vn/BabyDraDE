# Adding an Island Feature

Step-by-step guide for extending the Dynamic Island without inventing new
plumbing. Follow [dynamic-island-architecture.md](./dynamic-island-architecture.md)
for the invariants.

## 1. When to add a feature

Add a feature when it needs to occupy the notch capsule with its own view.
Do **not** add a feature for a pure background service with no visual state;
put that in `libs/services/` or use an existing indicator feature.

## 2. Create the module

```text
center/features/<name>/
├── mod.rs          # struct + IslandFeature impl (keep <= 200 lines)
├── service.rs      # one read() or data helpers (no GTK wiring)
├── controller/     # optional: keyboard.rs / pointer.rs
└── ui/             # optional: layout + render helpers
```

`mod.rs` template:

```rust
//! <Name> indicator feature for the Dynamic Island.

pub mod service;

use std::time::Duration;

use gtk4::prelude::*;

use crate::center::poller::spawn_poller;
use crate::center::{IslandCtx, IslandFeature, NotchWidget, CAPSULE_HEIGHT, CAPSULE_WIDTH};

pub const PRIORITY: u32 = 90;
pub const SHOW_DURATION: Duration = Duration::from_secs(2);

pub struct MyFeature {
    widgets: NotchWidget,
}

impl MyFeature {
    pub fn new() -> Self {
        Self {
            widgets: NotchWidget::with_value("my-icon", "My feature", ""),
        }
    }
}

impl Default for MyFeature {
    fn default() -> Self {
        Self::new()
    }
}

impl IslandFeature for MyFeature {
    fn id(&self) -> &'static str {
        "my_feature"
    }

    fn priority(&self) -> u32 {
        PRIORITY
    }

    fn size(&self) -> (i32, i32) {
        (CAPSULE_WIDTH, CAPSULE_HEIGHT)
    }

    fn is_alive(&self) -> bool {
        false // ephemeral: alive only while its timeout is unexpired
    }

    fn build(&mut self, ctx: &IslandCtx) -> gtk4::Widget {
        let widgets = self.widgets.clone();
        let ctx = ctx.clone();

        spawn_poller("island-my-feature", Duration::from_millis(1000), service::read,
            move |value| {
                widgets.update("my-icon", &value.to_string());
                ctx.override_show_for(SHOW_DURATION);
                ctx.tick();
            });

        self.widgets.container.clone().upcast()
    }
}
```

Rules for `build`:
- Wire **all** handlers here (click, keyboard, scroll, listeners, IPC). There
  is no later `init`/`attach` phase.
- Capture `ctx.clone()` in callbacks; never store a handle in the struct.
- Return the root widget; the engine wraps it in a hidden container.

## 3. Export and register

`features/mod.rs`:

```rust
pub mod my_feature;
pub use my_feature::MyFeature;
```

`center/registry.rs` — append exactly one line:

```rust
engine.register_feature(MyFeature::new());
```

That is the whole wiring. Do not touch `capsule.rs`, `engine.rs`, `ipc.rs` or
`ui/` for a normal feature.

## 4. Optional: modal popover + keyboard

```rust
let popover = MyPopover::new(&ctx.capsule()); // wraps IslandPopover::new_modal
let popover_rc = self.popover.clone();
let ctx_for_keys = ctx.clone();

ctx.attach_keyboard(&popover.popover, &popover.popover_box, move || {
    keyboard_controller(popover_rc.clone(), ctx_for_keys.clone())
});

fn on_click(&mut self) {
    if let Some(popover) = self.popover.borrow().as_ref() {
        popover.toggle();
    }
}
```

`IslandCtx::attach_keyboard` adds the factory to the popover, its content box,
the capsule and the layer window in one call. The layer keyboard mode is
switched by `ui/popover/lifecycle.rs` while a modal popover is mapped.

Override `is_popover_open()` so arbitration never hijacks an open popover:

```rust
fn is_popover_open(&self) -> bool {
    self.popover.borrow().as_ref().is_some_and(|p| p.is_visible())
}
```

## 5. Optional: IPC trigger

Add a method to the `org.babydra.Island` interface in `services/ipc.rs`, map it
to a topic string, and register inside `build`:

```rust
ctx.on_trigger(|| {
    ctx_for_trigger.override_show_for(Duration::from_secs(5));
    ctx_for_trigger.tick();
    // show popover ...
});
```

The topic is the feature's `id()`. Never create another D-Bus server or a
feature-local trigger thread-local.

## 6. Optional: stateful indicator

If the view must stay visible while active, override `is_alive()`:

```rust
fn is_alive(&self) -> bool {
    self.active.get()
}
```

Use `hover_keep()` only when the view should survive pointer hover, and
`capsule_class()` for feature-specific CSS (e.g. `Some("active-music")`).

## 7. Checklist

- [ ] `IslandFeature::build(ctx)` implemented; no old `widget/init/attach` hooks.
- [ ] No handle stored in the feature struct; all requests via `ctx`.
- [ ] Background work uses `spawn_poller` + `service::read()`.
- [ ] Popovers use `IslandPopover::new_slide`/`new_modal`.
- [ ] Keyboard uses `ctx.attach_keyboard`.
- [ ] IPC uses `ctx.on_trigger` only.
- [ ] One line added to `registry.rs`; no other framework file changed.
- [ ] Every file <= 200 lines; `cargo check -p babydra-topbar --offline` clean.
