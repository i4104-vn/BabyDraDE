//! Paths and typed load/save helpers for the split BabyDraDE configuration.
//!
//! Main settings live in `~/.babydra/babydra.conf`:
//! - `1st`
//! - `[autostart]`, `[power]`, `[wallpaper]`, `[notification]`
//! - any unknown top-level keys the user keeps there
//!
//! Subsystem settings live in small files under `~/.babydra/configs/`:
//! - `display.toml`    `[display]`
//! - `desktop.toml`    `[desktop]`
//! - `lockscreen.toml` `[lockscreen]`
//! - `clipboard.toml`  `[clipboard]`
//! - `blur.toml`       `[blur]`
//! - `cursor.toml`     `[cursor]`
//! - `input.toml`      `[mouse]`, `[touchpad]`, `[gestures]`
//! - `keymap.toml`     `[shortcuts]`

mod paths;
mod sections;
mod setup;

pub use paths::*;
pub use sections::*;
pub use setup::ensure_babydra_dirs;
