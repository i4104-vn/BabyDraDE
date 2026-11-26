# Architecture

BabyDraDE is a Wayland compositor built on Smithay 0.7 and GTK4, running nested
in a Winit window or directly on a TTY through Udev/DRM. The `window-manager`
crate is a **modular monolith**: independent feature modules over a small Clean
Architecture foundation.

## 1. Module map

```text
src/
├── main.rs, lib.rs, shortcuts.rs
├── shared/   types, ports, config models (common kernel)
├── core/     pure business rules (shortcuts, policies)
├── app/      bootstrap, wiring, runtime, state facade
├── infra/    Winit/Udev backends, config loaders, process launcher
├── protocol/ compositor-side bus module (router, connection, socket)
└── modules/
    ├── blur/      GPU blur module
    ├── input/     device events
    ├── render/    frame composition
    ├── shell/     launcher, switcher, lock, screenshot
    ├── wayland/   protocol runtime and delegates
    └── window/    registry, ops, hit_test, interaction, animation, titlebar
```

Every feature module has the same internal shape (`mod.rs` plus topic
sub-folders), owns its state and rendering, and communicates with other modules
only through ports in `shared`. Files stay under 500 lines; related files are
grouped by feature rather than left loose in one folder.

There is no standalone `protocol` crate. The bus is split by ownership:
`window_manager::protocol` (`src/protocol/`) holds the compositor-side `Router`,
`Connection`, socket helpers and `BusSender`; `common::protocol` holds the shared
wire types and codec; `services::protocol` holds the feature-side `Bus` client,
so features never depend on the compositor. The router is a pure state machine
over uid-scoped abstract Unix sockets with retained topics, and the compositor
hosts it as socket-driven calloop sources — frame composition never polls
protocol traffic. No files or queue directories are used.

## 2. Boundaries

| Area | Role | Must not |
| --- | --- | --- |
| `shared` | Common kernel: value types, ports, config models | Contain feature behaviour |
| `core` | Pure rules and policies | Import Smithay/GTK, own feature state |
| `app` | Bootstrap, DI wiring, runtime lifecycle, `BabydraState` facade | Make feature decisions |
| `infra` | Backends, config loaders, process launch | Decide business logic |
| `modules/*` | Feature state, logic and rendering | Import another feature's internals |

`BabydraState` lives in `app/state.rs` as a facade over seven components:
`WaylandRuntime`, `WindowRegistry`, `InputState`, `OverlayState`,
`ClipboardState`, `RenderState` and `ConfigSnapshot`.

## 3. Ports

`shared/ports.rs` holds every boundary trait. The input seam is split by
interface segregation (`WindowControl`, `HitTestQuery`, `InteractionBuffer`,
`SystemActions`, `OverlayBridge`, `SeatSink`) plus the toolkit-neutral
`QueryText` port; `InputContext` remains as a compatibility supertrait.
`BabydraState`'s implementations are explicit `impl` blocks in `app/state.rs`.

## 4. Frame loop and shell

`infra/frame_pacing.rs` owns the shared `begin_frame` / `send_frames` /
`end_frame` steps used by `infra/winit.rs` and `infra/udev/render.rs`. Window
animation and titlebar rendering live in `modules/window/{animation,titlebar}/`; GPU blur is
the standalone `modules/blur/` module. Each shell overlay owns its state,
input, layout and drawing under `modules/shell/<feature>/`.

## 5. Error strategy

- **Startup:** fail fast with a message naming the resource and context.
- **Runtime events:** never panic; recover poisoned locks, log absent seat
  capabilities and client dispatch failures.
- **Best effort:** every ignored result is explained or explicitly handled.

## 6. Further reading

- [LAYERS.md](LAYERS.md) — module map, boundaries, size and naming rules
- [DEVELOPMENT.md](DEVELOPMENT.md) — checks, CI and test matrix
- [DECISIONS.md](DECISIONS.md) — rationale behind the current structure
- [protocol README](../crates/window-manager/src/protocol/README.md) — router,
  abstract sockets, retained topics and the WM message contract
