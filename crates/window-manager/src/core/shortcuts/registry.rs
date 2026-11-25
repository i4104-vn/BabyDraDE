//! Runtime shortcut registry shared by the public API and the keyboard path.
//!
//! Config-driven shortcuts are replaced wholesale on reload; app-owned
//! registrations live here so they survive keymap reloads and can be
//! added, replaced or removed independently by their owner.

use std::collections::BTreeMap;
use std::fmt;
use std::sync::RwLock;

use common::{ShortcutBinding, ShortcutManifest, ShortcutTarget};

use crate::core::shortcuts::domain::{parse_combo, parse_modifiers, to_keysym, Mods};

/// Failure returned while registering or resolving a shortcut.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistrationError {
    /// The combo is malformed or has no modifier.
    InvalidCombo(String),
    /// The command is empty.
    InvalidTarget(String),
    /// The public API was called before the registry was published.
    NotReady,
}

impl fmt::Display for RegistrationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidCombo(combo) => write!(f, "invalid shortcut combo: {combo:?}"),
            Self::InvalidTarget(message) => write!(f, "invalid shortcut target: {message}"),
            Self::NotReady => f.write_str("shortcut registry is not ready"),
        }
    }
}

impl std::error::Error for RegistrationError {}

#[derive(Debug, Clone)]
struct Entry {
    binding: ShortcutBinding,
    mods: Mods,
    keysym: u32,
}

/// Thread-safe, owner-scoped registry of runtime shortcut bindings.
#[derive(Debug, Default)]
pub struct ShortcutRegistry {
    entries: RwLock<BTreeMap<(String, String), Entry>>,
}

impl ShortcutRegistry {
    /// Creates an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds or replaces one binding for `owner`.
    ///
    /// # Arguments
    ///
    /// * `owner` - App id that owns the binding.
    /// * `binding` - Binding to register; its `id` is the upsert key.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the combo is valid, otherwise a [`RegistrationError`].
    pub fn register(&self, owner: &str, binding: ShortcutBinding) -> Result<(), RegistrationError> {
        let entry = resolve(&binding)?;
        let mut entries = self.entries.write().unwrap_or_else(|e| e.into_inner());
        entries.insert((owner.to_string(), binding.id.clone()), entry);
        Ok(())
    }

    /// Removes one binding by owner and id.
    ///
    /// # Returns
    ///
    /// `true` when a binding was removed.
    pub fn unregister(&self, owner: &str, id: &str) -> bool {
        self.entries
            .write()
            .unwrap_or_else(|e| e.into_inner())
            .remove(&(owner.to_string(), id.to_string()))
            .is_some()
    }

    /// Removes every binding owned by `owner`.
    ///
    /// # Returns
    ///
    /// The number of bindings removed.
    pub fn unregister_owner(&self, owner: &str) -> usize {
        let mut entries = self.entries.write().unwrap_or_else(|e| e.into_inner());
        let before = entries.len();
        entries.retain(|(registered_owner, _), _| registered_owner != owner);
        before - entries.len()
    }

    /// Removes owners that no longer publish a manifest.
    ///
    /// # Arguments
    ///
    /// * `owners` - Owner ids still present in the shared directory.
    ///
    /// # Returns
    ///
    /// The number of bindings removed.
    pub fn retain_owners(&self, owners: &[String]) -> usize {
        let mut entries = self.entries.write().unwrap_or_else(|e| e.into_inner());
        let before = entries.len();
        entries.retain(|(owner, _), _| owners.contains(owner));
        before - entries.len()
    }

    /// Replaces all bindings for the manifest's owner in one atomic step.
    ///
    /// Invalid bindings are logged and skipped instead of failing the whole
    /// snapshot, so a single bad entry cannot silence an app's other keys.
    ///
    /// # Arguments
    ///
    /// * `manifest` - Fresh snapshot published by one app.
    ///
    /// # Returns
    ///
    /// The number of bindings applied.
    pub fn sync_owner(&self, manifest: ShortcutManifest) -> usize {
        let mut resolved = Vec::new();
        for binding in manifest.bindings {
            match resolve(&binding) {
                Ok(entry) => resolved.push(((manifest.owner.clone(), binding.id.clone()), entry)),
                Err(e) => tracing::warn!(
                    "ignoring invalid shortcut {:?} from {}: {}",
                    binding.id,
                    manifest.owner,
                    e
                ),
            }
        }

        let mut entries = self.entries.write().unwrap_or_else(|e| e.into_inner());
        entries.retain(|(owner, _), _| owner != &manifest.owner);
        let applied = resolved.len();
        entries.extend(resolved);
        applied
    }

    /// Returns the target bound to an exact key press.
    ///
    /// # Arguments
    ///
    /// * `mods` - Active modifier mask.
    /// * `keysym` - Layout-independent keysym.
    ///
    /// # Returns
    ///
    /// The first matching target, or `None` when nothing matches.
    pub(crate) fn target_for(&self, mods: Mods, keysym: u32) -> Option<ShortcutTarget> {
        let entries = self.entries.read().unwrap_or_else(|e| e.into_inner());
        entries
            .values()
            .find(|entry| entry.binding.enabled && entry.mods == mods && entry.keysym == keysym)
            .map(|entry| entry.binding.target.clone())
    }

    /// Returns every registered binding with its owner.
    ///
    /// # Returns
    ///
    /// A snapshot ordered by owner and binding id.
    pub fn bindings(&self) -> Vec<(String, ShortcutBinding)> {
        self.entries
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .iter()
            .map(|((owner, _), entry)| (owner.clone(), entry.binding.clone()))
            .collect()
    }
}

/// Validates a binding and resolves its combo into a mask and keysym.
fn resolve(binding: &ShortcutBinding) -> Result<Entry, RegistrationError> {
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

#[cfg(test)]
mod tests {
    use common::{KeyboardAction, ShortcutBinding, ShortcutManifest, ShortcutTarget};

    use super::{RegistrationError, ShortcutRegistry};
    use crate::shared::types::keysyms::{KEY_e, KEY_F1, KEY_t};
    use crate::core::shortcuts::domain::Mods;

    const KEY_F4: u32 = KEY_F1 + 3;

    fn binding(id: &str, combo: &str, target: ShortcutTarget) -> ShortcutBinding {
        ShortcutBinding {
            id: id.to_string(),
            combo: combo.to_string(),
            target,
            enabled: true,
        }
    }

    fn win() -> Mods {
        Mods {
            win: true,
            ..Mods::default()
        }
    }

    fn win_shift() -> Mods {
        Mods {
            win: true,
            shift: true,
            ..Mods::default()
        }
    }

    fn alt() -> Mods {
        Mods {
            alt: true,
            ..Mods::default()
        }
    }

    #[test]
    fn registered_command_matches_only_its_exact_combo() {
        let registry = ShortcutRegistry::new();
        registry
            .register(
                "app",
                binding(
                    "terminal",
                    "W-S-t",
                    ShortcutTarget::Command("kitty".into()),
                ),
            )
            .unwrap();

        assert_eq!(
            registry.target_for(win_shift(), KEY_t),
            Some(ShortcutTarget::Command("kitty".into()))
        );
        assert_eq!(
            registry.target_for(win(), KEY_t),
            None,
            "modifier mismatch must not match"
        );
        assert_eq!(
            registry.target_for(win_shift(), KEY_e),
            None,
            "key mismatch must not match"
        );
    }

    #[test]
    fn register_accepts_both_command_and_action_targets() {
        let registry = ShortcutRegistry::new();
        registry
            .register(
                "app",
                binding(
                    "terminal",
                    "W-e",
                    ShortcutTarget::Command("kitty".into()),
                ),
            )
            .unwrap();
        registry
            .register(
                "app",
                binding(
                    "power",
                    "A-F4",
                    ShortcutTarget::Action(KeyboardAction::ToggleLauncher),
                ),
            )
            .unwrap();

        assert_eq!(
            registry.target_for(win(), KEY_e),
            Some(ShortcutTarget::Command("kitty".into()))
        );
        assert_eq!(
            registry.target_for(alt(), KEY_F4),
            Some(ShortcutTarget::Action(KeyboardAction::ToggleLauncher))
        );
    }

    #[test]
    fn disabled_binding_never_matches() {
        let registry = ShortcutRegistry::new();
        let mut disabled = binding("terminal", "W-e", ShortcutTarget::Command("kitty".into()));
        disabled.enabled = false;
        registry.register("app", disabled).unwrap();

        assert_eq!(registry.target_for(win(), KEY_e), None);
        assert_eq!(registry.bindings().len(), 1);
    }

    #[test]
    fn registering_same_owner_and_id_replaces_binding() {
        let registry = ShortcutRegistry::new();
        registry
            .register(
                "app",
                binding("terminal", "W-e", ShortcutTarget::Command("kitty".into())),
            )
            .unwrap();
        registry
            .register(
                "app",
                binding(
                    "terminal",
                    "W-e",
                    ShortcutTarget::Action(KeyboardAction::Screenshot),
                ),
            )
            .unwrap();

        assert_eq!(registry.bindings().len(), 1);
        assert_eq!(
            registry.target_for(win(), KEY_e),
            Some(ShortcutTarget::Action(KeyboardAction::Screenshot))
        );
    }

    #[test]
    fn invalid_combos_are_rejected_at_registration() {
        let registry = ShortcutRegistry::new();
        let command = || ShortcutTarget::Command("kitty".into());

        for combo in ["e", "-e", "W-", "X-e", "W-unknown"] {
            assert!(
                matches!(
                    registry.register("app", binding("bad", combo, command())),
                    Err(RegistrationError::InvalidCombo(_))
                ),
                "combo {combo:?} should be rejected"
            );
        }
        assert!(registry.bindings().is_empty());
    }

    #[test]
    fn empty_command_is_rejected() {
        let registry = ShortcutRegistry::new();
        let result = registry.register(
            "app",
            binding("empty", "W-e", ShortcutTarget::Command("  ".into())),
        );
        assert!(matches!(result, Err(RegistrationError::InvalidTarget(_))));
    }

    #[test]
    fn sync_owner_replaces_snapshot_and_skips_bad_entries() {
        let registry = ShortcutRegistry::new();
        registry
            .register(
                "app",
                binding("old", "W-e", ShortcutTarget::Command("old".into())),
            )
            .unwrap();

        let applied = registry.sync_owner(ShortcutManifest {
            owner: "app".into(),
            revision: 1,
            bindings: vec![
                binding("good", "A-F4", ShortcutTarget::Action(KeyboardAction::Screenshot)),
                binding("bad", "nonsense", ShortcutTarget::Command("x".into())),
            ],
        });

        assert_eq!(applied, 1);
        assert_eq!(registry.bindings().len(), 1);
        assert_eq!(registry.target_for(win(), KEY_e), None);
        assert_eq!(
            registry.target_for(alt(), KEY_F4),
            Some(ShortcutTarget::Action(KeyboardAction::Screenshot))
        );
    }

    #[test]
    fn unregister_owner_and_retain_owners_remove_only_targeted_bindings() {
        let registry = ShortcutRegistry::new();
        for owner in ["topbar", "desktop"] {
            registry
                .register(
                    owner,
                    binding("key", "W-e", ShortcutTarget::Command(owner.into())),
                )
                .unwrap();
        }

        assert_eq!(registry.unregister_owner("topbar"), 1);
        assert_eq!(registry.bindings().len(), 1);
        assert_eq!(
            registry.target_for(win(), KEY_e),
            Some(ShortcutTarget::Command("desktop".into()))
        );

        assert_eq!(registry.retain_owners(&[]), 1);
        assert!(registry.bindings().is_empty());
    }
}
