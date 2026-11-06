# BabyDraDE

> **A lightweight, modular floating Wayland Desktop Environment built in Rust.**

BabyDraDE renders floating application windows with full server-side decorations (titlebar, control buttons, focused/unfocused borders, and resize handles) on top of a full-screen canvas background.

---

## Architecture

BabyDraDE is organized as a Cargo workspace adhering to Clean Architecture principles:

- **`crates/domain`**: Entities (`Window`, `Workspace`, `Output`, `Cursor`), Value Objects (`Rect`, `Point`, `Size`, `Color`, `KeyCombo`), Placement & MRU Focus rules, and Traits. Zero external runtime dependencies.
- **`crates/application`**: Services (`WindowManager`, `WorkspaceManager`, `InputDispatcher`, `AppLauncherService`), Commands, and Events.
- **`crates/presentation`**: Canvas background, Titlebar decorations (`[_]` Minimize, `[□]` Maximize, `[×]` Close), Borders, Resize handles, and multi-layer Scene Graph.
- **`crates/infrastructure`**: Wayland display server, `me.babydra.compositor` socket, XDG shell & seat handlers, Winit/DRM backends, Texture Cache, and GL Renderer.
- **`crates/drade-ctl`**: CLI control utility for IPC scripting.

---

## Quick Start

### Build & Run
```bash
# Build all crates
cargo build

# Run unit and integration tests
cargo test --workspace

# Start the compositor (dev mode)
cargo run
```

### Launch Wayland Clients
When BabyDraDE is running, launch apps pointing to its Wayland socket:
```bash
WAYLAND_DISPLAY=me.babydra.compositor alacritty
```

---

## Default Keybindings

| Shortcut | Action |
| :--- | :--- |
| `Super + Return` | Launch default terminal (`alacritty`) |
| `Super + Q` | Close focused window |
| `Super + F` | Toggle fullscreen |
| `Super + M` | Toggle maximize |
| `Super + H` | Minimize window |
| `Super + Tab` | Cycle window focus (MRU) |
| `Super + 1..4` | Switch to workspace 1..4 |
| `Super + Shift + 1..4` | Move focused window to workspace 1..4 |
| `Super + D` | Launch application finder |
| `Super + Shift + Q` | Quit compositor |

---

## License

MIT
