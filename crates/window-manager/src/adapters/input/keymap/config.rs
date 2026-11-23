//! Window-manager-owned keymap configuration.

use super::SharedShortcuts;
use crate::domain::shortcuts::Shortcut;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration;

const DEFAULT_KEYMAP: &str = include_str!("keymap.toml");
const WATCH_INTERVAL: Duration = Duration::from_secs(2);

#[derive(Deserialize, Default)]
struct ConfigFile {
    #[serde(default)]
    shortcuts: HashMap<String, String>,
}

/// Loads global shortcuts from `~/.babydra/configs/keymap.toml`.
pub(crate) fn load() -> Vec<Shortcut> {
    let path = services::config::get_keymap_path();
    export_default_if_missing(&path);
    let mut shortcuts = Vec::new();

    let content = match std::fs::read_to_string(&path) {
        Ok(content) => content,
        Err(e) => {
            tracing::warn!("cannot read keymap config {}: {}", path.display(), e);
            return shortcuts;
        }
    };

    let config = match toml::from_str::<ConfigFile>(&content) {
        Ok(config) => config,
        Err(e) => {
            tracing::warn!("cannot parse keymap config {}: {}", path.display(), e);
            return shortcuts;
        }
    };

    for (combo, command) in config.shortcuts {
        let Some((modifiers, key)) = split_combo(&combo) else {
            tracing::warn!("ignoring invalid keymap combo {:?}", combo);
            continue;
        };
        if command.trim().is_empty() {
            tracing::warn!("ignoring empty command for combo {:?}", combo);
            continue;
        }
        shortcuts.push(Shortcut {
            id: shortcuts.len() + 1,
            modifiers,
            key,
            command,
            enabled: true,
        });
    }

    tracing::info!("loaded {} keymap shortcut(s)", shortcuts.len());
    shortcuts
}

/// Splits `"W-S-t"` into `("W-S", "t")`. A key without modifiers is not a
/// valid global shortcut and is rejected.
fn split_combo(combo: &str) -> Option<(String, String)> {
    let (modifiers, key) = combo.rsplit_once('-')?;
    if key.is_empty() {
        return None;
    }
    Some((modifiers.to_string(), key.to_string()))
}

/// Watches the keymap file and replaces the shared table when it changes.
pub(crate) fn spawn_watcher(shared: SharedShortcuts) {
    if let Err(e) = std::thread::Builder::new()
        .name("babydra-keymap-watch".to_string())
        .spawn(move || {
            let path = services::config::get_keymap_path();
            let mut last_mtime = file_mtime(&path);

            loop {
                std::thread::sleep(WATCH_INTERVAL);
                let current = file_mtime(&path);
                if current.is_some() && current != last_mtime {
                    last_mtime = current;
                    tracing::info!("keymap config changed, reloading shortcuts");
                    *shared.write().unwrap() = load();
                }
            }
        })
    {
        tracing::error!("failed to start keymap watcher: {}", e);
    }
}

fn file_mtime(path: &std::path::Path) -> Option<std::time::SystemTime> {
    std::fs::metadata(path).and_then(|m| m.modified()).ok()
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

#[cfg(test)]
#[path = "../../../tests/adapters/input/keymap/config.rs"]
mod tests;
