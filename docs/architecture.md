# BabyDraDE — Architecture

BabyDraDE is a modern Wayland Desktop Environment built with Smithay 0.7.0 and GTK4.
It supports both direct TTY scanout (Udev/DRM backend) and nested execution (Winit backend).

## Workspace Layout

The workspace is cleanly divided into executables (`crates/`) and libraries (`libs/`):

### Executables (`crates/` with `src/main.rs`)
- `crates/window-manager`: Smithay Wayland compositor and window manager. Manages the Wayland display, socket, seat, outputs, rendering loop, and layer/XDG shell handlers, autostarts widgets, and captures screenshots directly through its renderer (with the annotation editor).
- `crates/features/desktop`: Desktop wallpaper/grid layer-shell app (`babydra-desktop`).
- `crates/features/topbar`: Top bar layer-shell app (`babydra-topbar`).
- App launching is handled by the compositor-owned launcher (`Super+Space`) in `crates/window-manager/src/adapters/desktop/launcher/` — a vertically centred icon bar on the left (no separate launcher executable).

### Libraries (`libs/` with `src/lib.rs`)
- `libs/services`: Core OS integration, hardware readers (battery, disk, CPU, RAM, network, volume, VPN), desktop configuration, themes, tray, and i18n localization. Configuration is split into `~/.babydra/babydra.conf` (main settings) and small files under `~/.babydra/configs/`; see [Configuration](configuration.md).
- `libs/ui-kit`: Reusable GTK4 design system, components, icons, theme engine, and stylesheets.

## `window-manager` — Clean Architecture

The crate follows Clean Architecture. Dependencies point **inward** only:

```text
   composition  (entry point + wiring)
        │
        ▼
   infrastructure  (frameworks & drivers: Smithay backends, GTK, process I/O)
        │
        ▼
   adapters       (interface adapters: Wayland protocol, input, rendering, desktop, window)
        │
        ▼
    application    (use cases + ports)
         │
         ▼
    domain         (entities and pure rules)
```

Inner layers never import an outer layer. `domain` is a leaf: it owns its keysym
table, titlebar rules and shortcut matching; window metadata is a plain value
type and its cache lives in `adapters/window`. `application` owns the decisions:
which command a key press runs, which window action a gesture maps to, what
geometry a maximized window gets, which selection MIME to capture and how a
finger drag moves the switcher. Its ports are being migrated from plain function
types to traits that can carry errors and be replaced in tests; the composition
root injects concrete adapters for them.

Large areas keep their own sub-folder: files are split by responsibility rather
than merged into big modules. Unit tests are inline `#[cfg(test)]` modules next
to the code they cover; integration and architecture tests live in `tests/`.

```text
src/
├── composition/      # entry-point wiring: run(), bootstrap, subcommand dispatch
├── infrastructure/   # backends (Winit/Udev), platform (brand), process, config/
│   └── config/       # gesture policy, keymap, pointer and touchpad config
├── adapters/         # compositor, input, rendering, desktop and window adapters
│   ├── compositor/   # protocol handlers + runtime state, hit-test, clipboard
│   ├── input/        # keyboard, pointer, touch, touchpad; config now in infrastructure
│   ├── rendering/    # frame composition, elements, blur, titlebar, animation
│   ├── desktop/      # launcher, switcher, lock, screenshot, clipboard picker
│   └── window/       # shared window registry, metadata and cursor mapping
├── application/      # clipboard, gestures, ports, shortcuts, switcher, windows
├── domain/           # keysyms, shortcuts, titlebar, window
├── tests/            # architecture gate (integration test)
└── main.rs           # composition root executable
resources/            # default keymap shipped with the compositor
```

Layer boundaries are enforced by code review, Rust module visibility
(`pub(crate)` inside a binary crate), and an architecture test that runs in CI.
The current import graph is pinned as a shrinking baseline while the remaining
cycles are removed; see [ADR 0004](adr/0004-architecture-gate.md). The original
decision and its rationale are recorded in
[ADR 0003](adr/0003-clean-architecture.md).

See [Animations, Constants and Enums](animations-constants-enums.md) for frame pacing,
[Configuration](configuration.md) for the config layout,
[Testing](testing.md) for the local checks, and the [ADRs](adr/) for decisions.
