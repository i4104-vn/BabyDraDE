//! Daemon orchestration for `babydra-keymap`.

use super::keyboard;
use super::mapping::Shortcut;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Type alias for the shared, hot-reloadable shortcut list.
pub type SharedShortcuts = Arc<RwLock<Vec<Shortcut>>>;

#[derive(Deserialize, Default)]
struct ConfigFile {
    #[serde(default)]
    shortcuts: HashMap<String, String>,
    #[serde(default)]
    custom_shortcuts: HashMap<String, String>,
}

/// Daemon entry point: loads config, starts hot-reload and keyboard listening.
pub async fn run() {
    let shortcuts: SharedShortcuts = Arc::new(RwLock::new(load()));

    // Hot-reload config in the background
    tokio::spawn(watch(shortcuts.clone()));

    tracing::info!("babydra-keymap daemon started");
    keyboard::run(shortcuts).await;
}

/// Loads all valid global shortcuts, merging custom keymap with active feature shortcuts.
fn load() -> Vec<Shortcut> {
    let path = infrastructure::config::get_keymap_path();
    let mut shortcuts = Vec::new();

    if path.exists() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(config) = toml::from_str::<ConfigFile>(&content) {
                for (combo, cmd) in config.shortcuts.iter().chain(config.custom_shortcuts.iter()) {
                    let mut parts = combo.split('-');
                    let key = parts.next_back().unwrap_or_default().to_string();
                    let modifiers = parts.collect::<Vec<_>>().join("-");
                    shortcuts.push(Shortcut {
                        id: shortcuts.len() + 1,
                        modifiers,
                        key,
                        command: cmd.clone(),
                        enabled: true,
                    });
                }
            }
        }
    }

    // Default system shortcuts if none configured yet
    if shortcuts.is_empty() {
        let exe = std::env::current_exe()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| "babydrade".to_string());

        shortcuts.push(Shortcut {
            id: 1,
            modifiers: "W-S".to_string(),
            key: "s".to_string(),
            command: format!("{exe} __widget screenshot"),
            enabled: true,
        });
        shortcuts.push(Shortcut {
            id: 2,
            modifiers: "W".to_string(),
            key: "l".to_string(),
            command: format!("{exe} __widget lock"),
            enabled: true,
        });
    }

    shortcuts.retain(|s| !s.key.is_empty() && !s.command.trim().is_empty());
    tracing::info!("loaded {} shortcut(s)", shortcuts.len());
    shortcuts
}

/// Watches configuration files (~/.babydra/keymap.toml), hot-reloading on change.
async fn watch(shared: SharedShortcuts) {
    let path = infrastructure::config::get_keymap_path();
    let mut last_mtime = file_mtime(&path);

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        let cur_mtime = file_mtime(&path);
        if cur_mtime != last_mtime && cur_mtime.is_some() {
            last_mtime = cur_mtime;
            tracing::info!("keymap config changed, reloading shortcuts");
            *shared.write().unwrap() = load();
        }
    }
}

fn file_mtime(path: &std::path::Path) -> Option<std::time::SystemTime> {
    std::fs::metadata(path).and_then(|m| m.modified()).ok()
}
