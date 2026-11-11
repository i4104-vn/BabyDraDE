//! Keymap daemon orchestration for the window manager.

use super::keyboard;
use super::mapping::Shortcut;
use std::sync::{Arc, RwLock};

/// Type alias for the shared, hot-reloadable shortcut list.
pub type SharedShortcuts = Arc<RwLock<Vec<Shortcut>>>;

/// Daemon entry point: loads config, starts hot-reload and keyboard listening.
pub async fn run() {
    let shortcuts: SharedShortcuts = Arc::new(RwLock::new(super::config::load()));

    // Hot-reload config in the background
    tokio::spawn(watch(shortcuts.clone()));

    tracing::info!("window-manager keymap started");
    keyboard::run(shortcuts).await;
}

/// Watches configuration files (~/.babydra/keymap.toml), hot-reloading on change.
async fn watch(shared: SharedShortcuts) {
    let path = services::config::get_keymap_path();
    let mut last_mtime = file_mtime(&path);

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        let cur_mtime = file_mtime(&path);
        if cur_mtime != last_mtime && cur_mtime.is_some() {
            last_mtime = cur_mtime;
            tracing::info!("keymap config changed, reloading shortcuts");
            *shared.write().unwrap() = super::config::load();
        }
    }
}

fn file_mtime(path: &std::path::Path) -> Option<std::time::SystemTime> {
    std::fs::metadata(path).and_then(|m| m.modified()).ok()
}
