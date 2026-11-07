# BabyDraDE — Current MVP Architecture

BabyDraDE is a small nested Wayland compositor built with Smithay 0.7.0. It
runs as a Winit window inside an existing Linux graphical session.

## Runtime

1. The root binary initializes logging and reads an optional app argument.
2. The infrastructure crate creates the Wayland display, socket, seat, output,
   and XDG shell handlers.
3. The Winit backend renders the compositor space with a black background and
   forwards keyboard and pointer events.
4. The selected test app is launched with BabyDraDE's Wayland display.

## Code layout

- src/main.rs: command-line entry point.
- crates/infrastructure/src/compositor: Smithay state, Wayland handlers, and
  input routing.
- crates/infrastructure/src/backends/winit_backend.rs: nested output,
  rendering, redraws, and window events.
- crates/domain, crates/application, and crates/presentation: retained
  workspace layers, currently placeholders for later work.
- crates/drade-ctl: retained as a placeholder; IPC is not implemented.

Smithay supplies the Wayland protocol and renderer building blocks. BabyDraDE
currently maps XDG toplevels into a Smithay Space, focuses the launched
window, and routes input to its surface. Window decorations, a desktop web
surface, app discovery, multiple-app controls, XWayland, and a DRM/TTY backend
are outside this milestone.
