//! File-based shortcut registration bridge.
//!
//! Watches manifests published by feature apps and applies them to the
//! application-level registry consumed by the keyboard pipeline.
//!
//! Polls app manifests and reconciles them with the shortcut registry.

use std::sync::Arc;
use std::thread;
use std::time::Duration;

use common::ShortcutManifest;

use crate::core::shortcuts::app::ShortcutRegistry;

const POLL_INTERVAL: Duration = Duration::from_millis(300);

/// Applies currently published manifests and starts the background watcher.
///
/// # Arguments
///
/// * `registry` - Registry shared with the keyboard pipeline and public API.
pub(crate) fn spawn_watcher(registry: Arc<ShortcutRegistry>) {
    apply(&registry, &services::system::shortcuts::read_all());

    if let Err(e) = thread::Builder::new()
        .name("babydra-shortcut-watch".into())
        .spawn(move || {
            let mut known: Vec<ShortcutManifest> = Vec::new();
            loop {
                thread::sleep(POLL_INTERVAL);
                let current = services::system::shortcuts::read_all();
                if current == known {
                    continue;
                }
                apply(&registry, &current);
                known = current;
            }
        })
    {
        tracing::error!("failed to start shortcut watcher: {e}");
    }
}

/// Reconciles the registry with one complete snapshot of published manifests.
///
/// # Arguments
///
/// * `registry` - Registry to update.
/// * `manifests` - All manifests currently present in the runtime directory.
pub(super) fn apply(registry: &ShortcutRegistry, manifests: &[ShortcutManifest]) {
    let owners: Vec<String> = manifests.iter().map(|m| m.owner.clone()).collect();
    registry.retain_owners(&owners);
    for manifest in manifests {
        registry.sync_owner(manifest.clone());
    }
}

#[cfg(test)]
mod tests {
    use common::{KeyboardAction, ShortcutBinding, ShortcutManifest, ShortcutTarget};

    use super::apply;
    use crate::core::shortcuts::app::ShortcutRegistry;
    use crate::shared::types::keysyms::{KEY_e, KEY_F1};
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

    fn manifest(owner: &str, bindings: Vec<ShortcutBinding>) -> ShortcutManifest {
        ShortcutManifest {
            owner: owner.to_string(),
            revision: 1,
            bindings,
        }
    }

    #[test]
    fn apply_installs_published_bindings() {
        let registry = ShortcutRegistry::new();
        apply(
            &registry,
            &[manifest(
                "topbar",
                vec![binding(
                    "clipboard",
                    "W-e",
                    ShortcutTarget::Action(KeyboardAction::ToggleLauncher),
                )],
            )],
        );

        let mods = Mods {
            win: true,
            ..Mods::default()
        };
        assert!(registry.target_for(mods, KEY_e).is_some());
    }

    #[test]
    fn apply_removes_bindings_of_apps_that_stopped_publishing() {
        let registry = ShortcutRegistry::new();
        let first = manifest(
            "topbar",
            vec![binding(
                "key",
                "A-F4",
                ShortcutTarget::Action(KeyboardAction::Screenshot),
            )],
        );
        apply(&registry, std::slice::from_ref(&first));

        let mods = Mods {
            alt: true,
            ..Mods::default()
        };
        assert!(registry.target_for(mods, KEY_F4).is_some());

        apply(&registry, &[]);
        assert_eq!(registry.bindings().len(), 0);
    }

    #[test]
    fn apply_replaces_an_apps_previous_snapshot() {
        let registry = ShortcutRegistry::new();
        let mods = Mods {
            alt: true,
            ..Mods::default()
        };
        apply(
            &registry,
            &[manifest(
                "topbar",
                vec![binding(
                    "old",
                    "A-F4",
                    ShortcutTarget::Action(KeyboardAction::Screenshot),
                )],
            )],
        );

        apply(
            &registry,
            &[manifest(
                "topbar",
                vec![binding(
                    "new",
                    "A-F4",
                    ShortcutTarget::Action(KeyboardAction::ToggleLauncher),
                )],
            )],
        );

        assert_eq!(registry.bindings().len(), 1);
        assert_eq!(
            registry.target_for(mods, KEY_F4),
            Some(ShortcutTarget::Action(KeyboardAction::ToggleLauncher))
        );
    }
}
