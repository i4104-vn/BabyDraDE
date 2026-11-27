//! Resolved entries for runtime shortcut bindings.

use common::{ShortcutBinding, ShortcutTarget};

use crate::core::shortcuts::{parse_combo, parse_modifiers, to_keysym, Mods};

use super::RegistrationError;

#[derive(Debug, Clone)]
pub(super) struct Entry {
    pub(super) binding: ShortcutBinding,
    pub(super) mods: Mods,
    pub(super) keysym: u32,
}

/// Validates a binding and resolves its combo into a mask and keysym.
pub(super) fn resolve(binding: &ShortcutBinding) -> Result<Entry, RegistrationError> {
    let (modifiers, key) = parse_combo(&binding.combo)
        .ok_or_else(|| RegistrationError::InvalidCombo(binding.combo.clone()))?;
    let mods = parse_modifiers(&modifiers)
        .filter(|mods| *mods != Mods::default())
        .ok_or_else(|| RegistrationError::InvalidCombo(binding.combo.clone()))?;
    let keysym =
        to_keysym(&key).ok_or_else(|| RegistrationError::InvalidCombo(binding.combo.clone()))?;
    if let ShortcutTarget::Command(command) = &binding.target {
        if command.trim().is_empty() {
            return Err(RegistrationError::InvalidTarget("empty command".into()));
        }
    }
    Ok(Entry {
        binding: binding.clone(),
        mods,
        keysym,
    })
}
