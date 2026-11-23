//! Translates Smithay modifier state into the domain modifier set.

use smithay::input::keyboard::ModifiersState;

use crate::domain::shortcuts::Mods;

use super::super::InputContext;

/// Returns the command of the first shortcut whose key and modifiers match
/// the current key press, or `None` when no shortcut matched.
pub(crate) fn command_for(
    state: &impl InputContext,
    modifiers: &ModifiersState,
    keysym: u32,
) -> Option<String> {
    let mods = Mods {
        shift: modifiers.shift,
        ctrl: modifiers.ctrl,
        alt: modifiers.alt,
        win: modifiers.logo,
    };
    state.shortcut_command(mods, keysym)
}
