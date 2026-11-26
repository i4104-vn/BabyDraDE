# Configuration

BabyDraDE uses one main file for central settings and several small files for
individual subsystems. Everything lives under `~/.babydra/`.

## 1. Directory layout

```text
~/.babydra/
├── babydra.conf          # main configuration
├── configs/
│   ├── blur.toml         # [blur]
│   ├── clipboard.toml    # [clipboard]
│   ├── cursor.toml       # [cursor]
│   ├── desktop.toml      # [desktop]
│   ├── display.toml      # [display]
│   ├── desktop.json      # debounced desktop icon coordinates, not TOML
│   ├── input.toml        # [mouse], [touchpad], [gestures]
│   ├── keymap.toml       # [shortcuts]
│   └── lockscreen.toml   # [lockscreen]
├── cursor/               # cursor themes
├── logs/                 # application logs
└── wallpaper/            # user wallpapers
```

## 2. Main file

`babydra.conf` only contains central groups:

```toml
"1st" = false

[autostart]
commands = ["babydra-desktop", "babydra-topbar"]

[power]
profile = "balanced"
auto_saver_enabled = true
saver_threshold = 20
charge_limit = 80

[wallpaper]
current = "/home/user/.babydra/wallpaper/wallpaper.jpeg"
mode = "static"

[notification]
dnd = false
```

## 3. Input method

The compositor exports `GTK_IM_MODULE`, `QT_IM_MODULE`, `SDL_IM_MODULE`,
`INPUT_METHOD=fcitx` and `XMODIFIERS=@im=fcitx` to child processes. Values already
set by the user are preserved.

To use an input method, a fcitx5 daemon must run in the Wayland session. Add it
to autostart if the system does not already start it:

```toml
[autostart]
commands = ["babydra-desktop", "babydra-topbar", "fcitx5"]
```

Unknown keys are preserved during migration, but new subsystems should not be
added to the main file.

## 4. Subsystem files

Example `configs/input.toml`:

```toml
[mouse]
natural_scroll = false
speed = 1.0
middle_emulation = false

[touchpad]
tap_to_click = true
tap_drag = true
tap_drag_lock = false
button_map = "lrm"
dwt = true
natural_scroll = false
speed = 1.0

[gestures]
three_finger_swipe_window = true
swipe_threshold = 100.0
```

Each subsystem keeps its original section names (`[blur]`, `[cursor]`, and so
on), so existing configurations are easy to read and migrate.

## 5. Automatic migration

When `ensure_babydra_dirs()` runs at compositor, desktop or topbar startup:

1. Subsystem sections still inside an old `babydra.conf` move to the matching
   file in `configs/`.
2. If the target file already has the section, its values win and the legacy
   section is removed from `babydra.conf`.
3. `babydra.conf` is rewritten with only the central groups, plus unknown keys
   the user kept.
4. An old `~/.babydra/keymap.toml` is copied to `configs/keymap.toml` when the
   new file does not exist.
5. Missing subsystem files are created with defaults.

Existing user data in a target file is never overwritten.

## 6. API for developers

Use `services::config` instead of reading files directly:

- `load_babydra_config()` / `save_babydra_config()` — main configuration only.
- `load_<subsystem>_config()` / `save_<subsystem>_config()` — one small file.
- `invalidate_cache()` — drop all cached values and re-read from disk.
- `get_*_config_path()` — resolve a concrete path.

Reads and writes go through `config::store`, which caches by modification time
and writes atomically with a temporary file plus rename. Input hot paths consume
snapshots held by `ConfigSnapshot` and `InputState`; they never read the disk.

## 7. Monitor orientation

Orientation is configured per physical output in
`~/.babydra/configs/display.toml`. Use the connector name reported by the
compositor log or by `/sys/class/drm/` (`eDP-1`, `DSI-1`, `HDMI-A-1`, `DP-1`, …):

```toml
[display]
# Keep the fixed rotations below even when a sensor is present.
auto_rotate = false

[[display.monitors]]
name = "eDP-1"
orientation = "90"
enabled = true

[[display.monitors]]
name = "HDMI-A-1"
orientation = "normal"
enabled = true
```

Accepted values are `normal` (or `0`), `90`, `180`, `270`, `flipped`,
`flipped-90`, `flipped-180` and `flipped-270`. Unknown values and disabled
entries are ignored. An unconfigured portrait panel still falls back to a
quarter turn so it renders landscape.

The same file drives both stages of the session:

- **Greeter:** at startup the greeter finds the most recently modified
  `display.toml` (last logged-in user from `/etc/passwd` first, then any home
  under `/home`) and applies the rotations directly to cage over the
  `zwlr_output_manager_v1` Wayland protocol. No `wlr-randr` installation is
  needed.
- **Desktop:** the compositor applies the configured rotation when it creates
  each DRM output; the single-output nested Winit backend uses the first
  configured monitor.

Changes take effect on the next login/session start: outputs are configured
once when they are created.

No environment variables take part in orientation any more. The former
`BABYDRADE_TRANSFORM`, `BABYDRADE_ORIENTATION`, `BABYDRADE_TOUCH_TRANSFORM`,
`BABYDRADE_AUTO_ROTATE` and `BABYDRADE_AUTO_ROTATE_DEBUG` overrides, along with
the installer flags and the `03-orientation.sh` step, have been removed.

### Automatic rotation

Automatic rotation is **on by default whenever an orientation sensor is
present**. Convertible/tablet devices are detected through either raw IIO sysfs
accelerometers (`/sys/bus/iio/devices/iio:deviceN`) or `iio-sensor-proxy` on
D-Bus (`net.hadess.SensorProxy`). The compositor debounces the reading and
applies the new transform to every output; touch coordinates follow the same
rotation through in-memory state.

When auto-rotation is active it takes precedence over the fixed orientation:
the configured rotation is the baseline the sensor rotates from, so a
sideways-mounted panel still rotates correctly. Set `auto_rotate = false` to
keep the configured rotation permanently. On machines without a sensor the
feature is a no-op and the configured rotation is used as-is.
