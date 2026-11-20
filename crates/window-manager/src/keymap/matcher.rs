//! Matches the current key press against configured shortcuts.

use smithay::input::keyboard::ModifiersState;

use super::mapping::{Mods, ResolvedShortcut};
use super::SharedShortcuts;

impl From<&ModifiersState> for Mods {
    fn from(value: &ModifiersState) -> Self {
        Self {
            shift: value.shift,
            ctrl: value.ctrl,
            alt: value.alt,
            win: value.logo,
        }
    }
}

/// Returns the command of the first shortcut whose key and modifiers exactly
/// match the current key press, or `None` when no shortcut matched.
pub fn command_for(
    modifiers: &ModifiersState,
    keysym: u32,
    shortcuts: &SharedShortcuts,
) -> Option<String> {
    let current = Mods::from(modifiers);

    shortcuts
        .read()
        .unwrap()
        .iter()
        .filter_map(ResolvedShortcut::resolve)
        .find(|s| s.keysym == keysym && s.mods == current)
        .map(|s| s.command)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keymap::mapping::Shortcut;
    use smithay::input::keyboard::keysyms;
    use std::sync::{Arc, RwLock};

    fn fixture() -> SharedShortcuts {
        Arc::new(RwLock::new(vec![Shortcut {
            id: 1,
            modifiers: "W-A".into(),
            key: "e".into(),
            command: "emacs".into(),
            enabled: true,
        }]))
    }

    #[test]
    fn matches_exact_modifiers_and_keysym() {
        let mods = ModifiersState {
            logo: true,
            alt: true,
            ..Default::default()
        };
        assert_eq!(
            command_for(&mods, keysyms::KEY_e, &fixture()).as_deref(),
            Some("emacs")
        );
    }

    #[test]
    fn rejects_wrong_modifiers_or_key() {
        let mut mods = ModifiersState {
            logo: true,
            ..Default::default()
        };
        assert_eq!(command_for(&mods, keysyms::KEY_e, &fixture()), None);

        mods.alt = true;
        assert_eq!(command_for(&mods, keysyms::KEY_t, &fixture()), None);
    }
}
