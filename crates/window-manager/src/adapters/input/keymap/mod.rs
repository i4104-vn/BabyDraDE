//! Configuration adapter for global shortcuts.
//!
//! Reads `~/.babydra/configs/keymap.toml` and delegates matching and command
//! dispatch to the injected `ShortcutService` use case through the input
//! seam. The compositor owns the input devices (libinput grabs them on a TTY),
//! so a separate evdev listener can never see key presses;
//! [`crate::adapters::input::keyboard`] calls the matcher from this module for
//! every key press.

pub(crate) mod matcher;

use super::InputContext;

/// Hands a matched command to the injected launcher port.
pub(crate) fn spawn_command(state: &impl InputContext, command: String) {
    state.launch_command(command);
}
