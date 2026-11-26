# Modular monolith and module rules

`crates/window-manager/src/` is a modular monolith: independent feature modules
over a small Clean Architecture foundation.

```text
src/
├── main.rs, lib.rs, shortcuts.rs   entry points and public shortcut API
├── shared/    common kernel: types, boundary ports, config models
├── core/      pure business rules only (no framework, no feature state)
├── app/       bootstrap, wiring, event-loop runtime, state facade
├── infra/     Winit/Udev backends, config loaders, process launcher
├── protocol/  compositor-side bus (router, connection, socket, BusSender)
└── modules/   independent feature modules
    ├── blur/      GPU blur pipeline and shaders
    ├── auto_rotate/ driver abstraction/factory + sensor drivers
    ├── input/     keyboard, pointer, touchpad, touch
    ├── render/    frame composition
    ├── shell/     launcher, switcher, lock, screenshot
    ├── wayland/   protocol runtime and delegates
    └── window/    registry, ops, hit_test, interaction, animation, titlebar
```

## Boundaries

- `shared` is the only common kernel; feature modules never reach into another
  feature's internals — they communicate through ports in `shared`.
- `core` contains pure rules (`shortcut` parsing/matching, policies). It has no
  Smithay, GTK, filesystem or process code and no feature-specific state.
- `app` orchestrates: it owns bootstrap, wiring, runtime lifecycle and the
  `BabydraState` facade. Business decisions stay in `core`/feature modules.
- `infra` only translates hardware and IO; it must not decide business logic.
- `protocol/` is compositor-internal: it owns the `Router`, `Connection`, socket
  helpers and `BusSender`. Shared wire types and the codec live in
  `common::protocol`; the feature-side `Bus` client lives in
  `services::protocol`. There is no standalone `protocol` crate.
- Each module under `modules/` owns its state, logic and rendering, and uses the
  same internal shape: `mod.rs` plus topic sub-folders (`state/`, `render/`,
  `service/`, `ports/`). Animation and titlebar belong to `window`; blur is its
  own module.

## Size and naming

- Every `.rs` file is **under 500 lines**. Split by topic, never by wrapper.
- No `#[path]`, no `<name>.rs` beside `<name>/`.
- Do not leave many unrelated loose files in one folder; group them by feature.

`scripts/check-layers.sh` and `cargo test -p window-manager --test architecture`
enforce the structure.
