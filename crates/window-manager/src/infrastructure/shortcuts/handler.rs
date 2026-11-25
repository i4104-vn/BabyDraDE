//! Polls app manifests and reconciles them with the shortcut registry.

use std::sync::Arc;
use std::thread;
use std::time::Duration;

use common::ShortcutManifest;

use crate::application::shortcuts::ShortcutRegistry;

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
