# Runtime shortcut registration

BabyDraDE has three shortcut sources. They are checked in this order on every
key press:

1. **System bindings** — hardcoded in `adapters/input/keyboard/shortcuts.rs`
   (quit, lock, Alt+Tab, screenshot, ...). They cannot be remapped.
2. **App registrations** — published at runtime through the API described here.
3. **Keymap file** — `~/.babydra/configs/keymap.toml`, hot-reloaded every 2 s.

A registration survives keymap reloads because it lives in a separate
`ShortcutRegistry`, not in the config-driven table.

## Two APIs, one registry

| Caller | API | Crate |
| --- | --- | --- |
| Code linked into the compositor | `window_manager::shortcuts` | `window-manager` (lib target) |
| Feature apps in another process | `services::system::shortcuts` | `services` |

Both paths end up in the same `application::shortcuts::ShortcutRegistry`, so
matching and lifecycle rules are identical.

## Registering from another process

Each app owns one manifest in `$XDG_RUNTIME_DIR/babydra-shortcuts`. One file per
owner means apps never overwrite each other, and an atomic rename means the
compositor never reads a half-written file. The compositor watches the
directory every 300 ms and reconciles its registry with the published set.

```rust
use common::{ShortcutTarget, KeyboardAction};
use services::system::shortcuts::ShortcutClient;

let shortcuts = ShortcutClient::new("org.babydrade.topbar");

// A built-in compositor action.
shortcuts.bind(
    "launcher",
    "W-Space",
    ShortcutTarget::Action(KeyboardAction::ToggleLauncher),
)?;

// A shell command executed by the compositor.
shortcuts.bind(
    "clipboard",
    "W-v",
    ShortcutTarget::Command(
        "gdbus call --session --dest org.babydra.Island \
         --object-path /org/babydra/Island \
         --method org.babydra.Island.ToggleClipboard"
            .into(),
    ),
)?;

// Full lifecycle support.
shortcuts.unregister("launcher")?;      // remove one binding
shortcuts.clear()?;                     // remove every binding for this app
```

`id` is the per-app upsert key: registering the same id twice replaces the
old binding. `enabled = false` keeps a binding published but stops it matching.
Combos use the keymap format (`W` = Super, `A` = Alt, `C` = Ctrl, `S` = Shift)
and require at least one modifier. Invalid combos, empty commands and unparseable
key names are rejected at registration and logged by the compositor.

### Topbar shortcuts

The topbar used to install GTK accelerators (`<Super>v`, `<Super>r`,
`<Super>F4`) and relied on `KeyboardMode::OnDemand`. The compositor consumes
those key presses before the panel sees them, so the accelerators never fired.
`center/services/shortcuts.rs` now publishes the same combos through
`ShortcutClient`; the compositor owns the bindings and dispatches them to the
island's existing `org.babydra.Island` D-Bus methods, which already run the
feature callbacks. This requires `gdbus` and a session bus in the compositor
environment, as for any other command-target shortcut.

## Registering from inside the compositor

The composition root publishes the live registry at startup. Any in-process
component can then call:

```rust
use common::ShortcutTarget;
use window_manager::shortcuts as shortcuts_api;

shortcuts_api::bind(
    "babydra-internal",
    "rerender",
    "W-F6",
    ShortcutTarget::Command("babydra-window-manager --refresh".into()),
)?;

let active = shortcuts_api::bindings()?;
shortcuts_api::unregister("babydra-internal", "rerender")?;
```

Before startup these calls return `RegistrationError::NotReady`; after startup
they share the same state as the file-based API. `handle()` exposes the
`Arc<ShortcutRegistry>` for callers that need direct access.

## Lifecycle notes

- A failed manifest write leaves the previous file intact (write-then-rename).
- A malformed manifest is logged and skipped; other apps are unaffected.
- Removing an app's manifest removes all of its bindings on the next watcher
  tick. Apps should call `clear()` on shutdown, but a crash also converges.
- System bindings always win when an app registers the same combo.
