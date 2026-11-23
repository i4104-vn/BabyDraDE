//! Configuration adapter for global shortcuts.
//!
//! Reads `~/.babydra/configs/keymap.toml` and delegates matching and command
//! dispatch to the [`ShortcutRegistry`] use case. The compositor owns the
//! input devices (libinput grabs them on a TTY), so a separate evdev listener
//! can never see key presses; [`crate::adapters::input::keyboard`] calls the
//! matcher from this module for every key press.

pub(crate) mod config;
pub(crate) mod matcher;

use std::sync::{Arc, RwLock};

use crate::{application::shortcuts::ShortcutRegistry, domain::shortcuts::Shortcut};

/// Shortcut list shared with the hot-reload watcher.
pub(crate) type SharedShortcuts = Arc<RwLock<Vec<Shortcut>>>;

/// Reloads shortcuts from `~/.babydra/configs/keymap.toml` immediately.
pub(crate) fn reload() {
    ShortcutRegistry::get().reload();
}

/// Hands a matched command to the injected launcher port.
pub(crate) fn spawn_command(command: String) {
    ShortcutRegistry::get().launch(command);
}
