//! Thread-safe storage for runtime shortcut bindings.

use std::collections::BTreeMap;
use std::sync::RwLock;

use common::{ShortcutBinding, ShortcutManifest, ShortcutTarget};

use crate::core::shortcuts::Mods;

use super::entry::{resolve, Entry};
use super::RegistrationError;

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
