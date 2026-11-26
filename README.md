# BabyDraDE

BabyDraDE is a minimal Wayland compositor built on Smithay. The first milestone
starts in a nested Winit window, clears it to black, provides keyboard and mouse
input, and launches one Wayland app for a smoke run.

## Requirements

- Linux with a running graphical session (Wayland or X11)
- Rust stable
- Smithay's Linux build dependencies, including Wayland, XKB common, EGL, and GLES
- `wl-clipboard` (`wl-copy` / `wl-paste`) — installed automatically by `install.sh`
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

## Protocol bus

Features and the compositor exchange typed messages over a uid-scoped abstract
Unix socket — no files or queue directories are created. There is no standalone
`protocol` crate; the contract is split by ownership:

- `window_manager::protocol` (in `crates/window-manager/src/protocol/`) owns the
  compositor-side transport: `Router`, `Connection`, socket helpers and
  `BusSender`. The compositor hosts the router as socket-driven calloop sources,
  so frame composition never polls protocol traffic.
- `common::protocol` owns the shared wire types and codec (`Request`, `Response`,
  `Event`, `Envelope`, `MessageId`, `ProtocolError`, ...).
- `services::protocol` owns the feature-side `Bus` client that apps link against
  without depending on the compositor.

The router stamps sources, correlates requests, tracks presence, and retains
topic values for late subscribers until their TTL expires. Focused-window
metadata is a retained `window.active` event; window actions are requests to the
compositor's `wm` peer. See [docs/PROTOCOL.md](docs/PROTOCOL.md) for the wire
format, router rules, file layout and usage examples.

## File dialogs and `xdg-open`

The compositor binary provides the native GTK file chooser and an `xdg-open`
replacement. Apps can point `~/.local/bin/xdg-open` at the binary: with a path
or URL it opens the target via GIO/BabyDra apps, and with no target it shows the
chooser. See [docs/FILEDIALOG.md](docs/FILEDIALOG.md) for ownership and setup.

## Development

CI runs on every push and pull request (`.github/workflows/ci.yml`) and checks
layer boundaries, formatting, a full build, tests and clippy. Reproduce locally:

    scripts/check-layers.sh
    cargo fmt --all -- --check
    cargo check --workspace --all-targets
    cargo test --workspace
    cargo clippy --workspace --all-targets -- -D warnings

Layer and module rules are summarized in [docs/LAYERS.md](docs/LAYERS.md), with
more detail in [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) and
[docs/DEVELOPMENT.md](docs/DEVELOPMENT.md).

## Clipboard

`Ctrl+C`, `Ctrl+X` and `Ctrl+V` are always delivered to the focused application,
so normal copy/paste works as on any desktop. The compositor caches every
selection, so copied text survives the source app closing.

The greetd login screen lives in `crates/features/greeter` and is built as
`babydra-greeter`. The distro-provided `greetd.service` is enabled as the
display manager (the setup script only adds a small drop-in when the unit does
not already claim tty1). After a successful login the greeter starts
`BabyDra`, so the compositor comes up immediately after login.
Override the session with `BABYDRADE_GREETER_SESSION` if needed.

`install.sh` parses the command line and then runs the numbered steps in
`scripts/` in order: locale, GTK theme sync, dependency check (including greetd
and cage), hardware session setup, release build + system install, and greetd
service setup (display-manager alias, `graphical.target`,
tty1 takeover). It only installs and enables things — it never launches the
desktop session. greetd starts the compositor after login. Use `--skip-deps`
to bypass the dependency step.

The compositor creates an automatic Wayland socket in XDG_RUNTIME_DIR and
sets WAYLAND_DISPLAY for launched applications. It currently focuses the
single launched window and has no custom decorations, app menu, XWayland, or
direct DRM/TTY backend.
