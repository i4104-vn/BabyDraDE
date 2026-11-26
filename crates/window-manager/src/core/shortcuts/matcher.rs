//! Shortcut matching rules.

use super::{Mods, ResolvedShortcut, Shortcut};

/// Finds the command bound to an exact key press.
///
/// # Arguments
///
/// * `mods` - Modifier mask active during the key press.
/// * `keysym` - Layout-independent XKB keysym.
/// * `shortcuts` - Configured shortcuts, searched in order.
///
/// # Returns
///
/// The command of the first exact match, or `None` when nothing matches.
pub(crate) fn command_for(mods: Mods, keysym: u32, shortcuts: &[Shortcut]) -> Option<String> {
    shortcuts
        .iter()
        .filter_map(ResolvedShortcut::resolve)
        .find(|s| s.keysym == keysym && s.mods == mods)
        .map(|s| s.command)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::types::keysyms;

    #[test]
    fn matches_exact_modifiers_and_keysym() {
        let shortcuts = vec![Shortcut {
            id: 1,
            modifiers: "W-A".into(),
            key: "e".into(),
            command: "emacs".into(),
            enabled: true,
        }];
        let mods = Mods {
            win: true,
            alt: true,
            ..Mods::default()
        };
        assert_eq!(
            command_for(mods, keysyms::KEY_e, &shortcuts).as_deref(),
            Some("emacs")
        );
        assert_eq!(command_for(mods, keysyms::KEY_t, &shortcuts), None);
    }
}
