//! Window-manager-owned keymap configuration.

use super::mapping::Shortcut;
use serde::Deserialize;
use std::collections::HashMap;

const DEFAULT_KEYMAP: &str = include_str!("keymap.toml");

#[derive(Deserialize, Default)]
struct ConfigFile {
    #[serde(default)]
    shortcuts: HashMap<String, String>,
}

/// Loads global shortcuts from `~/.babydra/keymap.toml`.
pub fn load() -> Vec<Shortcut> {
    let path = services::config::get_keymap_path();
    export_default_if_missing(&path);
    let mut shortcuts = Vec::new();

    if let Ok(content) = std::fs::read_to_string(&path) {
        if let Ok(config) = toml::from_str::<ConfigFile>(&content) {
            for (combo, command) in config.shortcuts.iter() {
                let mut parts = combo.split('-');
                let key = parts.next_back().unwrap_or_default().to_string();
                let modifiers = parts.collect::<Vec<_>>().join("-");
                shortcuts.push(Shortcut {
                    id: shortcuts.len() + 1,
                    modifiers,
                    key,
                    command: command.clone(),
                    enabled: true,
                });
            }
        }
    }

    if shortcuts.is_empty() {
        shortcuts.extend([Shortcut {
            id: 1,
            modifiers: "W-S".to_string(),
            key: "s".to_string(),
            command: "babydra-screenshot".to_string(),
            enabled: true,
        }]);
    }

    shortcuts.retain(|shortcut| {
        !shortcut.key.is_empty() && !shortcut.command.trim().is_empty() && shortcut.enabled
    });
    tracing::info!("loaded {} shortcut(s)", shortcuts.len());
    shortcuts
}

/// Exports the bundled keymap template on first startup.
fn export_default_if_missing(path: &std::path::Path) {
    if path.exists() {
        return;
    }

    if let Some(parent) = path.parent() {
        if let Err(error) = std::fs::create_dir_all(parent) {
            tracing::warn!(
                "cannot create keymap config directory {}: {}",
                parent.display(),
                error
            );
            return;
        }
    }

    if let Err(error) = std::fs::write(path, DEFAULT_KEYMAP) {
        tracing::warn!(
            "cannot export default keymap to {}: {}",
            path.display(),
            error
        );
    } else {
        tracing::info!("exported default keymap to {}", path.display());
    }
}
