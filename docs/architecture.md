# BabyDraDE — Architecture

BabyDraDE is a modern Wayland Desktop Environment built with Smithay 0.7.0 and GTK4.
It supports both direct TTY scanout (Udev/DRM backend) and nested execution (Winit backend).

## Workspace Layout

The workspace is cleanly divided into executables (`crates/`) and libraries (`libs/`):

### Executables (`crates/` with `src/main.rs`)
- `crates/window-manager`: Smithay Wayland compositor and window manager. Manages the Wayland display, socket, seat, outputs, rendering loop, and layer/XDG shell handlers, autostarts widgets, and captures screenshots directly through its renderer (with the annotation editor).
- `crates/features/desktop`: Desktop wallpaper/grid layer-shell app (`babydra-desktop`).
- `crates/features/topbar`: Top bar layer-shell app (`babydra-topbar`).
- App launching is handled by the compositor-owned launcher (`Super+Space`) in `crates/window-manager/src/extensions/launcher/` — a vertically centred icon bar on the left (no separate launcher executable).

### Libraries (`libs/` with `src/lib.rs`)
- `libs/services`: Core OS integration, hardware readers (battery, disk, CPU, RAM, network, volume, VPN), desktop configuration, themes, tray, and i18n localization.
- `libs/ui-kit`: Reusable GTK4 design system, components, icons, theme engine, and stylesheets.
