//! Translates Smithay modifier state into the domain modifier set.

use smithay::input::keyboard::ModifiersState;

use crate::domain::shortcuts::Mods;

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
