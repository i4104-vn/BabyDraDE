//! Public in-process shortcut registration API.
//!
//! The composition root publishes the live registry at startup. Any component
//! linked into this crate can then call [`register`] (or [`bind`]) to add a
//! shortcut, [`unregister`] to remove one, and [`bindings`] to inspect the
//! current table. Apps in other processes should use the file-based client in
//! `services::system::shortcuts` instead.

use std::sync::{Arc, OnceLock};

pub use common::{ShortcutBinding, ShortcutTarget};

pub use crate::application::shortcuts::{RegistrationError, ShortcutRegistry};

static REGISTRY: OnceLock<Arc<ShortcutRegistry>> = OnceLock::new();

/// Publishes the live registry once, during composition.
///
/// # Arguments
///
/// * `registry` - Registry shared with the keyboard pipeline.
pub(crate) fn publish(registry: Arc<ShortcutRegistry>) {
    let _ = REGISTRY.set(registry);
}

/// Returns the published registry, or a `NotReady` error before startup.
fn registry() -> Result<&'static Arc<ShortcutRegistry>, RegistrationError> {
    REGISTRY.get().ok_or(RegistrationError::NotReady)
}

/// Returns a clone of the published registry handle, when available.
///
/// # Returns
///
/// The shared registry, or `None` before the compositor is wired up.
pub fn handle() -> Option<Arc<ShortcutRegistry>> {
    REGISTRY.get().cloned()
}

/// Adds or replaces one shortcut for `owner`.
///
/// # Arguments
///
/// * `owner` - Stable id of the app owning the binding.
/// * `binding` - Binding to register; its `id` is the upsert key.
///
/// # Returns
///
/// `Ok(())` when the binding is valid and the registry is ready.
pub fn register(owner: &str, binding: ShortcutBinding) -> Result<(), RegistrationError> {
    registry()?.register(owner, binding)
}

/// Adds or replaces one shortcut from its parts.
///
/// # Arguments
///
/// * `owner` - Stable id of the app owning the binding.
/// * `id` - Stable per-app shortcut id.
/// * `combo` - Combo in keymap format, for example `W-S-t`.
/// * `target` - Command or compositor action to run.
///
/// # Returns
///
/// `Ok(())` when the binding is valid and the registry is ready.
pub fn bind(
    owner: &str,
    id: &str,
    combo: &str,
    target: ShortcutTarget,
) -> Result<(), RegistrationError> {
    register(
        owner,
        ShortcutBinding {
            id: id.to_string(),
            combo: combo.to_string(),
            target,
            enabled: true,
        },
    )
}

/// Removes one shortcut by owner and id.
///
/// # Returns
///
/// `true` when a binding was removed.
pub fn unregister(owner: &str, id: &str) -> Result<bool, RegistrationError> {
    Ok(registry()?.unregister(owner, id))
}

/// Removes every shortcut owned by `owner`.
///
/// # Returns
///
/// The number of bindings removed.
pub fn unregister_owner(owner: &str) -> Result<usize, RegistrationError> {
    Ok(registry()?.unregister_owner(owner))
}

/// Returns every registered binding with its owner.
///
/// # Returns
///
/// A snapshot ordered by owner and binding id.
pub fn bindings() -> Result<Vec<(String, ShortcutBinding)>, RegistrationError> {
    Ok(registry()?.bindings())
}
