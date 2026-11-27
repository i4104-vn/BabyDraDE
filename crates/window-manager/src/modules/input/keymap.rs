//! Configuration adapter for global shortcuts.
//!
//! Reads `~/.babydra/configs/keymap.toml` and delegates matching and command
//! dispatch to the injected `ShortcutService` use case through the input
//! seam. The compositor owns the input devices (libinput grabs them on a TTY),
//! so a separate evdev listener can never see key presses;
//! [`crate::modules::input::keyboard`] calls the matcher from this module for
//! every key press.

pub(crate) use super::InputContext;

/// Hands a matched command to the injected launcher port.
pub(crate) fn spawn_command(state: &impl InputContext, command: String) {
    state.launch_command(command);
}

pub(crate) mod matcher {
    //! Translates Smithay modifier state into the domain modifier set.

    use smithay::input::keyboard::ModifiersState;

    use crate::core::shortcuts::Mods;

    use super::super::InputContext;

    /// Converts Smithay's modifier state into the framework-free modifier mask.
    ///
    /// # Arguments
    ///
    /// * `modifiers` - Modifier state reported with the key event.
    ///
    /// # Returns
    ///
    /// The equivalent domain modifier mask.
    pub(crate) fn to_mods(modifiers: &ModifiersState) -> Mods {
        Mods {
            shift: modifiers.shift,
            ctrl: modifiers.ctrl,
            alt: modifiers.alt,
            win: modifiers.logo,
        }
    }

    /// Returns the command of the first shortcut whose key and modifiers match
    /// the current key press, or `None` when no shortcut matched.
    pub(crate) fn command_for(
        state: &impl InputContext,
        modifiers: &ModifiersState,
        keysym: u32,
    ) -> Option<String> {
        state.shortcut_command(to_mods(modifiers), keysym)
    }
}
