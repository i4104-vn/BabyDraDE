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
   adapters       (interface adapters: Wayland protocol, input, rendering, shell)
        │
        ▼
    application    (use cases + ports)
         │
         ▼
    domain         (entities and pure rules)
```

Inner layers never import an outer layer. `domain` is a leaf: it owns its keysym
table, window metadata cache, titlebar/cursor rules and shortcut matching, and
depends on no config file, process, protocol handle or toolkit. `application`
owns the decisions: which command a key press runs, which window action a
gesture maps to, what geometry a maximized window gets, which selection MIME to
capture and how a finger drag moves the switcher. Its ports are plain function
types (`LaunchCommand`, `LoadShortcuts`, `LoadGesturePolicy`) plus the
framework-free `GesturePolicy` snapshot; the composition root injects concrete
adapters for them.

Large areas keep their own sub-folder: files are split by responsibility rather
than merged into big modules. Unit tests live in `src/tests/`, mirroring the
source tree, and are attached with `#[path]` so they stay module-local.

```text
src/
├── composition/      # entry-point wiring: run(), bootstrap, subcommand dispatch
├── infrastructure/   # backends (Winit/Udev), platform (brand), config, process
├── adapters/         # compositor, input, rendering and shell interface adapters
│   ├── compositor/   # protocol handlers + state/ (windows, shell, hit-test,
│   │                 #   clipboard, metadata, titlebar, ...) — one concern per file
│   ├── input/        # keyboard, keymap config adapter, pointer, touch, touchpad
│   ├── rendering/    # frame composition, elements, blur, titlebar, animation
│   └── shell/        # launcher, switcher, lock, screenshot, clipboard
├── application/      # clipboard, gestures, ports, shortcuts, switcher, windows
├── domain/           # cursor, keysyms, shortcuts, titlebar, window
├── tests/            # mirrors the folders above, one file per source module
└── main.rs           # composition root executable
```

Layer boundaries are enforced by code review and Rust module visibility
(`pub(crate)` inside a binary crate); there is no separate fitness-function
tooling. The decision and its rationale are recorded in
[ADR 0003](adr/0003-clean-architecture.md).

See [Animations, Constants and Enums](animations-constants-enums.md) for frame pacing,
[Configuration](configuration.md) for the config layout,
[Testing](testing.md) for the local checks, and the [ADRs](adr/) for decisions.
