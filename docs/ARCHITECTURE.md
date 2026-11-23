# Architecture

BabyDraDE is a Wayland compositor built on Smithay 0.7 and GTK4. It runs both
nested in a Winit window and directly on a TTY through Udev/DRM. The
`window-manager` crate follows Clean Architecture: dependencies point inward,
and the inner layers contain the decisions while adapters only translate.

## 1. Dependency rule

```text
composition → infrastructure → adapters → application → domain
```

Inner layers never import an outer layer. In addition, the adapter area follows
a one-way order of its own:

```text
window → input → rendering → desktop → compositor (Wayland facade)
```

`adapters/window` is the neutral layer: it owns window records, metadata and the
`InputContext` seam, and imports none of the other adapter areas. `adapters/input`
reaches desktop actions through that seam instead of importing `adapters/desktop`.
`adapters/rendering` consumes visual snapshots and never calls back into the
compositor. `adapters/compositor` is the highest adapter: it assembles the
others and owns the Wayland protocol handlers.

The architecture test pins the remaining six one-way edges as a shrinking
baseline. There are no cycles. See [DEVELOPMENT.md](DEVELOPMENT.md#architecture-gate).

## 2. Layers

| Layer | Role | May import |
| --- | --- | --- |
| `domain/` | Pure entities and rules: `geometry`, `keysyms`, `shortcuts`, `titlebar`, `window` | nothing inside the crate |
| `application/` | Use cases and ports: `autostart`, `clipboard`, `gestures`, `shortcuts`, `switcher`, `windows`, `window_action` | `domain` |
| `adapters/` | Protocol, input, rendering, desktop overlays and window registry | `application`, `domain`, other adapters in the allowed order |
| `infrastructure/` | Backends, config loaders, process launcher, platform helpers | `application` ports and the compositor runtime |
| `composition/` | Coordinator, wiring, runtime lifecycle, bootstrap, commands | everything |

`domain` and `application` contain no Smithay, GTK, Wayland protocol, filesystem
or process code. Geometry is defined locally in `domain/geometry.rs`; adapters
convert to and from Smithay's `Point`, `Size` and `Rectangle` at the boundary.

## 3. State ownership

`BabydraState` used to be a 39-field god object with `impl` blocks scattered
across ten files. It is now a facade over seven components:

| Component | Owns |
| --- | --- |
| `WaylandRuntime` | Display globals, loop signal, seat, data device, popup manager, socket |
| `WindowRegistry` | Managed windows in Z order, active window, exit detection |
| `InputState` | Move/resize state, pointer and touch buffers, gesture accumulators |
| `OverlayState` | Launcher and Alt+Tab switcher state |
| `ClipboardState` | Pending MIME lists and capture channel state |
| `RenderState` | Blur engine and shared render resources |
| `ConfigSnapshot` | Gesture policy and cached natural-scroll settings |

The facade still carries nine compatibility fields that `composition` and the
backends read directly. A method that changes data must live on the component
that owns the data; adding a new `impl BabydraState` outside `state.rs` requires
a deliberate decision.

## 4. Ports and dependency injection

Effectful boundaries are traits that carry structured errors:

| Port | Error | Purpose |
| --- | --- | --- |
| `ShortcutRepository` | `ConfigError` | Load the shortcut table; reload keeps the previous snapshot on failure |
| `CommandLauncher` | `LaunchError` | Run a command matched by a shortcut |
| `AutostartLauncher` | `String` | Spawn one autostart command on the compositor display |
| `LoadGesturePolicy` | — | Function alias at the adapter boundary; still to be trait-ified |

The composition root creates the implementations once and injects them as
`Arc<dyn Trait + Send + Sync>`. There is no global registry, no `OnceLock` and no
service locator in `application`. Pure policies such as `maximize_target`,
`scrub_index` and MIME preference remain plain functions and are tested directly.

## 5. Input seam

`adapters/window/input_context.rs` defines a narrow `InputContext` trait: window
queries, hit testing, transient interaction state, configuration snapshots,
overlay commands and seat dispatch. `BabydraState` implements it in
`adapters/compositor/state/input_context.rs`. Input adapters depend only on the
trait, so they can be read and tested without knowing the protocol runtime.

All input-to-desktop effects (lock screen, clipboard picker, screenshot) are
methods on this trait. A single `WindowRegistry::active_window(&Space)` resolver
is used by handlers and input code instead of ad-hoc `space.elements().next_back()`.

## 6. Frame loop

`infrastructure/backends/render_loop.rs` owns the steps shared by Winit and
Udev/DRM: `begin_frame`, `send_frames` and `end_frame`. Each backend only handles
its own surface, renderer and presentation details. Frame pacing comes from
`animation::FRAME_INTERVAL`; Udev schedules from the previous deadline so a slow
frame cannot drift the cadence. See [ANIMATIONS.md](ANIMATIONS.md#frame-pacing).

## 7. Error strategy

- **Startup:** fail fast with a message that names the resource and its context.
- **Runtime events:** never panic. Poisoned locks are recovered with
  `unwrap_or_else(|poisoned| poisoned.into_inner())`; absent seat capabilities and
  client dispatch failures are logged and skipped.
- **Best effort:** a `let _ =` must be either explained by a comment or a debug
  log, or replaced with explicit error handling.

There are no `unwrap()` calls left in the input and compositor runtime paths.
Five `expect` calls remain: four at startup and one in a Smithay callback whose
API returns a reference with no fallback value.

## 8. Known debt

1. Nine flat compatibility fields on `BabydraState`; the target is a facade of
   seven components only.
2. Four `impl BabydraState` blocks for cross-component work (hit testing,
   switcher, sizing, animation) still need signature changes to move.
3. `LoadGesturePolicy` is still a function alias; screenshot requests are still
   made by the backends directly to the desktop adapter.
4. Two shortcut systems coexist: hardcoded system bindings and the configurable
   keymap. System bindings currently run first.
5. `libs/common` mixes domain enums with design tokens.

Crate splitting (`babydra-wm-core`, `babydra-wm-wayland`, `babydra-wm-render`,
`babydra-desktop`) is deliberately deferred until the items above are closed.
Splitting earlier would turn architectural debt into build debt.

## 9. Further reading

- [SUMMARY.md](SUMMARY.md) — Vietnamese summary of the completed refactor
- [DEVELOPMENT.md](DEVELOPMENT.md) — checks, CI, testing and smoke matrix
- [DECISIONS.md](DECISIONS.md) — the decisions behind the current structure
