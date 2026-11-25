//! Shared keyboard-shortcut registration protocol.
//!
//! Feature apps publish [`ShortcutManifest`] values through the file-based
//! client in `services::system::shortcuts`; the compositor reads the very same
//! types when it matches a key press. Keeping the protocol here (rather than
//! inside either process) means both sides agree on combo and target encoding.

use serde::{Deserialize, Serialize};

use crate::KeyboardAction;

/// Effect produced when a registered shortcut matches a key press.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShortcutTarget {
    /// Run a shell command through the compositor's command launcher.
    Command(String),
    /// Execute a built-in compositor action.
    Action(KeyboardAction),
}

/// One shortcut owned by an app.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShortcutBinding {
    /// Stable per-app id, used to replace or remove this binding.
    pub id: String,
    /// Combo in keymap format, for example `W-S-t` or `A-F4`.
    pub combo: String,
    /// Effect to run when the combo is pressed.
    pub target: ShortcutTarget,
    /// Disabled bindings remain published but never match.
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

/// One app's published shortcut table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShortcutManifest {
    /// Owner id, usually the app id (for example `org.babydrade.topbar`).
    pub owner: String,
    /// Incremented on every mutation so watchers can detect changes reliably.
    pub revision: u64,
    /// All shortcuts currently owned by the app.
    pub bindings: Vec<ShortcutBinding>,
}
