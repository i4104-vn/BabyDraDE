//! Configuration adapters.
//!
//! All filesystem access, default export, watcher threads and parsing live
//! here. Adapters receive ready-made snapshots instead of reading config.

pub(crate) use gesture::gesture_policy;

pub(crate) mod gesture {
    //! Configuration adapter — translates services config models into core policies.

    use crate::shared::ports::GesturePolicy;

    /// Converts the persisted gesture model into the framework-free policy.
    ///
    /// # Arguments
    ///
    /// * `cfg` - Gesture configuration loaded from disk.
    ///
    /// # Returns
    ///
    /// The policy snapshot consumed by the gesture use case.
    impl From<&services::models::GestureConfig> for GesturePolicy {
        fn from(cfg: &services::models::GestureConfig) -> Self {
            Self {
                swipe_threshold: cfg.swipe_threshold,
                four_finger_swipe_threshold: cfg.four_finger_swipe_threshold,
                touch_swipe_threshold: cfg.touch_swipe_threshold,
                touch_tap_timeout_ms: cfg.touch_tap_timeout_ms,
                three_finger_swipe_window: cfg.three_finger_swipe_window,
                three_finger_swipe_up_maximize: cfg.three_finger_swipe_up_maximize,
                three_finger_swipe_down_minimize: cfg.three_finger_swipe_down_minimize,
                four_finger_swipe_window: cfg.four_finger_swipe_window,
                four_finger_swipe_up_fullscreen: cfg.four_finger_swipe_up_fullscreen,
                four_finger_swipe_down_close: cfg.four_finger_swipe_down_close,
                touch_two_finger_swipe_window: cfg.touch_two_finger_swipe_window,
                touch_two_finger_swipe_down_minimize: cfg.touch_two_finger_swipe_down_minimize,
                touch_three_finger_tap: cfg.touch_three_finger_tap,
            }
        }
    }

    /// Reloads gesture settings from disk.
    ///
    /// # Returns
    ///
    /// A fresh policy snapshot for the compositor state.
    pub(crate) fn gesture_policy() -> GesturePolicy {
        GesturePolicy::from(&services::config::load_gesture_config())
    }
}

pub(crate) mod keymap {
    //! Keymap loading and hot-reload.
    //!
    //! Keymap configuration models.

    use std::{
        collections::HashMap,
        sync::{Arc, RwLock},
    };

    use crate::core::shortcuts::{parse_combo, Shortcut};
    use crate::shared::ports::{ConfigError, ShortcutRepository};
    use serde::Deserialize;

    use std::time::Duration;

    const DEFAULT_KEYMAP: &str = include_str!("../../resources/keymap.toml");
    const WATCH_INTERVAL: Duration = Duration::from_secs(2);

    /// Shortcut table shared with the hot-reload watcher.
    pub(crate) type SharedShortcuts = Arc<RwLock<Vec<Shortcut>>>;

    #[derive(Deserialize, Default)]
    pub(super) struct ConfigFile {
        #[serde(default)]
        pub(super) shortcuts: HashMap<String, String>,
    }

    /// Loads global shortcuts from `~/.babydra/configs/keymap.toml`.
    pub(crate) fn load() -> Vec<Shortcut> {
        let path = services::config::get_keymap_path();
        export_default(&path);
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
            let Some((modifiers, key)) = parse_combo(&combo) else {
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

    /// Exposes the keymap loader through the application's `ShortcutRepository`
    /// port. The loader degrades gracefully (it logs and returns an empty table
    /// when the file is missing or malformed), so no failure needs wrapping today;
    /// `ConfigError` keeps the port ready for future fallible loaders.
    #[derive(Debug, Default)]
    pub(crate) struct KeymapRepository;

    impl ShortcutRepository for KeymapRepository {
        fn load(&self) -> Result<Vec<Shortcut>, ConfigError> {
            Ok(load())
        }
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
    fn export_default(path: &std::path::Path) {
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
    mod tests {
        use super::*;

        #[test]
        fn parses_a_real_config() {
            let raw = r#"
        [shortcuts]
        "W-A-e" = "emacs"
        "W-S-t" = "kitty"
        "bad" = "ignored"
        "W-X" = ""
        "#;
            let config: ConfigFile = toml::from_str(raw).unwrap();
            assert_eq!(config.shortcuts.len(), 4);

            let mut shortcuts = Vec::new();
            for (combo, command) in config.shortcuts {
                if let Some((modifiers, key)) = parse_combo(&combo) {
                    if !command.trim().is_empty() {
                        shortcuts.push(Shortcut {
                            id: shortcuts.len() + 1,
                            modifiers,
                            key,
                            command,
                            enabled: true,
                        });
                    }
                }
            }
            assert_eq!(shortcuts.len(), 2);
            let combos: Vec<_> = shortcuts
                .iter()
                .map(|s| (s.modifiers.as_str(), s.key.as_str()))
                .collect();
            assert!(combos.contains(&("W-A", "e")));
            assert!(combos.contains(&("W-S", "t")));
        }
    }
}

pub(crate) mod snapshot {
    //! Cached configuration snapshot used by the input pipeline.
    //!
    //! The model itself belongs to the application layer; it is re-exported here
    //! so `shared::config::snapshot::ConfigSnapshot` keeps resolving.
    pub(crate) use crate::app::models::state::ConfigSnapshot;
}
