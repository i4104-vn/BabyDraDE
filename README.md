# BabyDraDE

BabyDraDE is a minimal Wayland compositor built on Smithay. The first milestone
starts in a nested Winit window, clears it to black, provides keyboard and mouse
input, and launches one Wayland app for a smoke run.

## Requirements

- Linux with a running graphical session (Wayland or X11)
- Rust stable
- Smithay's Linux build dependencies, including Wayland, XKB common, EGL, and GLES
- `wl-clipboard` (`wl-copy` / `wl-paste`) — installed automatically by `start.sh`
- A Wayland app to launch via a configurable shortcut

## Run

    cargo run

Launch a terminal (or any app) with a configurable shortcut in
`~/.babydra/configs/keymap.toml`:

    [shortcuts]
    "W-S-t" = "foot"

Change the command to `alacritty`, `kitty`, or any other executable. Changes
reload within 2 seconds.

The full config layout (main `babydra.conf` plus split files in `configs/`) is
documented in [docs/configuration.md](docs/configuration.md).

## Clipboard

`Ctrl+C`, `Ctrl+X` and `Ctrl+V` are always delivered to the focused application,
so normal copy/paste works as on any desktop. The compositor caches every
selection, so copied text survives the source app closing.

The greetd login screen lives in `crates/features/greeter` and is built as
`babydra-greeter`. `packaging/systemd/greetd.service` runs greetd at boot (as
the display manager); after a successful login the greeter starts
`babydra-window-manager`, so the compositor comes up immediately after login.
Override the session with `BABYDRADE_GREETER_SESSION` if needed.

`start.sh` parses the command line and then runs the numbered steps in
`scripts/` in order: locale, GTK theme sync, orientation, dependency check
(including greetd + cage), session setup, release build + system install,
greetd service setup (display-manager alias, `graphical.target`, tty1 takeover),
and finally launch. The compositor is always launched from the installed binary
on `PATH`, never from `target/` in the source tree. Use `--skip-deps` to bypass
the dependency step.

The compositor creates an automatic Wayland socket in XDG_RUNTIME_DIR and
sets WAYLAND_DISPLAY for launched applications. It currently focuses the
single launched window and has no custom decorations, app menu, XWayland, or
direct DRM/TTY backend.
