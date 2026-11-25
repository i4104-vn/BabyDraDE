# Dynamic Island Architecture

Defines the single, uniform structure of the topbar Dynamic Island in
`crates/features/topbar/src/center/`. It is a port of `babydra-island` adapted
to BabyDraDE's `services`, `ui-kit` and `animation-gtk`.

## 1. Golden Rules

1. **One API**: every feature implements `IslandFeature::build(&mut self, ctx: &IslandCtx)`.
   There is no `widget()` + `init()` + `attach()` trio and no compatibility
   bridge/adapter layer between the engine and features.
2. **One context**: `IslandCtx` is the only object a feature receives. It owns
   the view handle, capsule, immediate tick, popover dismissal, keyboard
   controller attachment and IPC trigger registration.
3. **One registration point**: `center/registry.rs` is the only place that
   calls `engine.register_feature(...)`.
4. **One IPC service**: `center/services/ipc.rs` owns D-Bus (`org.babydra.Island`).
   Features call `ctx.on_trigger(...)`; they never create their own D-Bus server
   or trigger thread-local.
5. **One poller**: `center/services/poller.rs::spawn_poller` is used by every
   background indicator. Each feature's `service.rs` only exposes a `read()`.
6. **One widget kit**: `center/ui/notch.rs` + `center/ui/popover.rs` provide the
   capsule and popover bases. Features compose them; they do not reimplement
   slide animations or keyboard-mode switching.
7. **Max 200 lines per file**. Split by responsibility, not by feature alone.

## 2. Directory Layout

```text
center/
├── mod.rs                  # Public exports only; no logic
├── registry.rs             # register_all(&mut IslandEngine) — the only registration point
├── engine/                 # Framework core (state machine only, no GTK feature code)
│   ├── mod.rs              # IslandEngine: register_feature, tick, click dispatch
│   ├── builder.rs          # Widget-tree construction (wrapper, brackets, capsule)
│   ├── feature.rs          # IslandFeature trait + capsule size constants
│   ├── context.rs          # IslandCtx (handle + shared services)
│   ├── handle.rs           # ViewState + IslandViewHandle (request/override/deadlines)
│   ├── record.rs           # Internal ViewRecord
│   └── arbitration.rs      # refresh_requests + select_winner (pure rules)
├── services/               # Infrastructure shared by all features
│   ├── mod.rs
│   ├── ipc.rs              # GIO D-Bus service + trigger topic registry
│   └── poller.rs           # Generic worker-thread poller -> GTK main context
├── ui/                     # Visual building blocks
│   ├── mod.rs
│   ├── capsule.rs          # IslandCapsule: engine owner, 150 ms tick, registration
│   ├── notch.rs            # NotchWidget: icon | title | value capsule layout
│   ├── popover/
│   │   ├── mod.rs          # IslandPopover: new_slide / new_modal + methods
│   │   ├── registry.rs     # Open-popover tracking, Escape, outside-click dismissal
│   │   ├── lifecycle.rs    # map/unmap: CSS class, focus, slide-in
│   │   └── animation.rs    # toggle + cancelable slide-out
│   └── transition.rs       # Expand/collapse capsule animations
└── features/               # One folder per feature
    ├── mod.rs              # Re-exports only
    ├── clipboard/{mod.rs, controller/{mod.rs,keyboard.rs,pointer.rs}, service.rs, ui/}
    ├── default/mod.rs
    ├── media/{mod.rs, model.rs, format.rs, poll.rs, popover.rs, visualizer.rs, ui/{render.rs,view.rs}}
    ├── power/{mod.rs, models/, controller/keyboard.rs, ui/}
    ├── recording/{mod.rs, models/, service.rs + service/, controller/actions/, ui/popover/}
    └── system/{mod.rs, <name>/{mod.rs, service.rs}}
```

## 3. Module Roles

| File | Responsibility | Must NOT contain |
| :--- | :--- | :--- |
| `engine/feature.rs` | Trait definition, size constants | Widget code, feature implementations |
| `engine/context.rs` | Context methods delegating to handle/services | Feature-specific widget logic |
| `engine/arbitration.rs` | Winner selection rules | Signal connections, D-Bus, GTK widgets |
| `services/ipc.rs` | D-Bus registration and topic dispatch | Feature behaviour |
| `services/poller.rs` | Thread + channel plumbing | Data parsing or UI updates |
| `ui/notch.rs` | Capsule presentation widget | State machines |
| `ui/popover/mod.rs` | Popover base API | Feature layouts |
| `features/<name>/mod.rs` | Struct + `IslandFeature` impl + orchestration | Duplicate handle/trigger plumbing |
| `features/<name>/service.rs` | One `read()` (or data helpers) | Keyboard/GTK wiring |
| `features/<name>/controller/` | Keyboard and pointer event controllers | D-Bus server setup |
| `features/<name>/ui/` | Layout and rendering helpers | Arbitration decisions |

## 4. Feature Lifecycle (the only flow)

```text
IslandCapsule::new
  → engine.set_self_weak(weak)
  → ipc::init()                       (once)
  → registry::register_all(&mut engine)
      → engine.register_feature(F::new())
          → ViewState + IslandViewHandle created by the engine
          → IslandCtx::new(...)        (handle, capsule, weak engine, feature id)
          → feature.build(&ctx)        (widget + listeners + popover + IPC trigger)
          → ViewRecord pushed
  → glib::timeout_add_local(150 ms, engine.tick)
      → arbitration::refresh_requests
      → feature.tick(&ctx) for every view
      → arbitration::select_winner
      → ui::transition::apply_transition
```

`IslandCtx::tick()` upgrades the weak engine reference and uses
`try_borrow_mut`, so an event callback can request arbitration without
re-entrant borrow panics.

## 5. Arbitration Order

1. A feature with an open popover always wins (never hijacked).
2. Otherwise the most recent explicit override (`override_show_for`) wins.
3. Otherwise highest `priority()`, tie-broken by most recent `show()` request.
4. Deadlines (`show_for` / `override_show_for`) are purged at each tick.
5. `hover_keep()` only keeps a view alive while the pointer is over the capsule.

## 6. Shared Services

### IPC trigger topics

`org.babydra.Island` at `/org/babydra/Island` exposes:

| Method | Topic | Feature |
| :--- | :--- | :--- |
| `ShowClipboard`, `ToggleClipboard` | `clipboard` | clipboard |
| `ShowPower`, `TogglePower` | `power` | power |
| `ShowRecording` | `recording` | recording |

`IslandCtx::on_trigger` registers the callback under the feature's `id()`.
GIO method calls arrive on a worker thread and are forwarded to the GTK main
context before the callbacks run.

### Poller contract

Each system feature exposes `service::read() -> T` where `T: Send + 'static`:

```rust
spawn_poller("island-volume", POLL_INTERVAL, service::read, move |state| {
    if changed {
        widgets.update(icon, &label);
        ctx.override_show_for(SHOW_DURATION);
        ctx.tick();
    }
});
```

The reader runs on a named worker thread; the callback always runs on the
GTK main context. Never create a bespoke thread/channel trio inside a feature.

## 7. Known Adaptations from `babydra-island`

| Upstream | BabyDraDE topbar |
| :--- | :--- |
| `zbus` D-Bus interface | `gio` low-level object registration (already a dependency) |
| `babydra_core` clipboard/hardware/recording services | `libs/services` equivalents |
| `babydra_ui_kit` | `libs/ui-kit` (`get_icon`, `create_popover`, `crop_circle`) |
| `babydra_core::services::recording` | `features/recording/service/recording.rs` adapter (idle until a recorder backend exists) |
| `island::default_island()` singleton | weak engine reference inside `IslandCtx` |

## 8. Validation

```bash
cargo check -p babydra-topbar --offline
cargo check --workspace --offline
```

Acceptance: zero warnings, no `bridge` module, no per-feature D-Bus server,
no per-feature handle storage, all files <= 200 lines.
