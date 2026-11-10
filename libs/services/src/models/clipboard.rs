//! Clipboard history and hotkey shortcut configuration models.

use serde::{Deserialize, Serialize};

fn default_true() -> bool {
    true
}

fn default_clipboard_max_items() -> usize {
    20
}

fn default_clipboard_modifiers() -> String {
    "W".to_string()
}

fn default_clipboard_key() -> String {
    "v".to_string()
}

/// Clipboard manager configuration persisted in `~/.babydra/babydra.conf` under `[clipboard]`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ClipboardConfig {
    /// Whether clipboard history tracking daemon is active.
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Maximum number of historic clipboard entries retained.
    #[serde(default = "default_clipboard_max_items")]
    pub max_items: usize,
    /// Modifier keys for the clipboard picker popup (e.g. `"W"` for Super/Windows key).
    #[serde(default = "default_clipboard_modifiers")]
    pub shortcut_modifiers: String,
    /// Trigger key name for the clipboard picker (e.g. `"v"`).
    #[serde(default = "default_clipboard_key")]
    pub shortcut_key: String,
}

impl Default for ClipboardConfig {
    /// Returns default clipboard configuration.
    ///
    /// # Returns
    ///
    /// A new [`ClipboardConfig`] with 20 items and Super+V shortcut.
    fn default() -> Self {
        Self {
            enabled: true,
            max_items: default_clipboard_max_items(),
            shortcut_modifiers: default_clipboard_modifiers(),
            shortcut_key: default_clipboard_key(),
        }
    }
}
