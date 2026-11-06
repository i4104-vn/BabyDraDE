# BabyDraDE — Architecture Documentation

## 1. Overview

BabyDraDE is a lightweight floating Wayland Desktop Environment built in Rust using a clean 4-layer architecture.

```
┌────────────────────────────────────────────────────────┐
│                   BabyDraDE Compositor                 │
│                                                        │
│  Presentation Layer (Canvas + Titlebars + Borders)    │
│            ▲                                           │
│  Application Layer (WindowManager, Workspaces, Input)  │
│            ▲                                           │
│  Domain Layer (Entities, Value Objects, Rules, Traits) │
│            ▲                                           │
│  Infrastructure Layer (Smithay / Winit, DRM, GL)       │
└────────────────────────────────────────────────────────┘
```

## 2. Dependency Inversion Rules

- `domain` contains zero external runtime dependencies.
- `application` depends solely on `domain`.
- `presentation` depends on `domain` and `application`.
- `infrastructure` depends on `domain` and `application`.
- `babydrade` (root binary) brings all crates together and handles bootstrap & injection.

## 3. Layers Breakdown

### Domain Layer (`crates/domain`)
- **Entities:** `Window`, `Workspace`, `Output`, `Cursor`.
- **Value Objects:** `Rect`, `Point`, `Size`, `Edges`, `Color`, `KeyCombo`.
- **Business Rules:** `CascadePlacement`, `CenterPlacement`, `MruFocusHistory`, `SizeConstraints`.
- **Traits:** `CompositorBackend`, `CanvasRenderer`, `InputSource`, `ConfigStore`, `AppRegistry`.

### Application Layer (`crates/application`)
- **Services:**
  - `WindowManager`: Coordinates open, close, move, resize, minimize, maximize, and MRU focus tracking.
  - `WorkspaceManager`: Multi-workspace routing (1..=4).
  - `InputDispatcher`: Dispatches keyboard shortcuts to commands.
  - `AppLauncherService`: Launches client apps with `WAYLAND_DISPLAY`.
- **Commands & Events:** Strongly typed CQRS-style commands (`WindowCommand`, `WorkspaceCommand`, `SystemCommand`) and events (`DesktopEnvironmentEvent`).

### Presentation Layer (`crates/presentation`)
- **Window Frame:** Server-side decorations including titlebar with action buttons (`[_]`, `[□]`, `[×]`), 4-direction borders, and 8-zone resize handles.
- **Canvas:** Desktop wallpaper and background surface rendering.
- **Scene Graph:** Layer-ordered pipeline (`Background` → `Windows` in z-order → `Overlay` → `Cursor`).

### Infrastructure Layer (`crates/infrastructure`)
- **Compositor:** Wayland display server, socket binding (`me.babydra.compositor`), global registration, XDG shell and seat handlers.
- **Backends:** `WinitBackend` (for development inside an OS window) and `DrmBackend` (bare-metal KMS).
- **Rendering:** `TextureCache` for zero-redundancy GPU buffer uploads, and `GlRenderer`.

