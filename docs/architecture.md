# BabyDraDE — Architecture

BabyDraDE is a modern Wayland Desktop Environment built with Smithay 0.7.0 and GTK4.
It supports both direct TTY scanout (Udev/DRM backend) and nested execution (Winit backend).

## Workspace Layout

The workspace is cleanly divided into executables (`crates/`) and libraries (`libs/`):

### Executables (`crates/` with `src/main.rs`)
- `crates/window-manager`: Smithay Wayland compositor and window manager. Manages the Wayland display, socket, seat, outputs, rendering loop, and layer/XDG shell handlers, autostarts widgets, and captures screenshots directly through its renderer (with the annotation editor).
- `crates/features/desktop`: Desktop wallpaper/grid layer-shell app (`babydra-desktop`).
- `crates/features/topbar`: Top bar layer-shell app (`babydra-topbar`).
- App launching is handled by the compositor-owned launcher (`Super+Space`) in `crates/window-manager/src/shell/launcher/` — a vertically centred icon bar on the left (no separate launcher executable).

### Libraries (`libs/` with `src/lib.rs`)
- `libs/services`: Core OS integration, hardware readers (battery, disk, CPU, RAM, network, volume, VPN), desktop configuration, themes, tray, and i18n localization. Configuration is split into `~/.babydra/babydra.conf` (main settings) and small files under `~/.babydra/configs/`; see [Configuration](configuration.md).
- `libs/ui-kit`: Reusable GTK4 design system, components, icons, theme engine, and stylesheets.

## `window-manager` module layers

```text
src/
├── app/          # composition root, env bootstrap, subcommand dispatch
├── backends/     # Winit / Udev adapters
├── compositor/   # Wayland protocol, space, seat and window state
├── input/        # keyboard, pointer, touch and touchpad pipelines
├── keymap/       # shortcut configuration and matching
├── model/        # plain shared types (leaf layer)
├── platform/     # brand, GTK helpers and future OS adapters
├── rendering/    # frame composition, elements, blur, titlebar, animation
└── shell/        # launcher, switcher, lock, screenshot, clipboard
```

Layer edges are enforced by `cargo xtask check` using
[`xtask/architecture.toml`](../xtask/architecture.toml). Existing violations
are listed in [`xtask/baseline.toml`](../xtask/baseline.toml) and documented in
[Architecture baseline](architecture-baseline.md).

See [Animations, Constants and Enums](animations-constants-enums.md) for frame pacing,
[Configuration](configuration.md) for the config layout,
[Testing](testing.md) for CI/gates, and the [ADRs](adr/) for decisions.
