# BabyDraDE

BabyDraDE is a minimal Wayland compositor built on Smithay. The first milestone
starts in a nested Winit window, clears it to black, provides keyboard and mouse
input, and launches one Wayland app for a smoke run.

## Requirements

- Linux with a running graphical session (Wayland or X11)
- Rust stable
- Smithay's Linux build dependencies, including Wayland, XKB common, EGL, and GLES
- A Wayland app to launch via a configurable shortcut

## Run

    cargo run

Launch a terminal (or any app) with a configurable shortcut in
`~/.babydra/keymap.toml`:

    [shortcuts]
    "W-S-t" = "foot"

Change the command to `alacritty`, `kitty`, or any other executable. Changes
reload within 2 seconds.

start.sh runs the same nested development compositor without installing system
packages or changing user permissions.

The compositor creates an automatic Wayland socket in XDG_RUNTIME_DIR and
sets WAYLAND_DISPLAY for launched applications. It currently focuses the
single launched window and has no custom decorations, app menu, XWayland, or
direct DRM/TTY backend.
