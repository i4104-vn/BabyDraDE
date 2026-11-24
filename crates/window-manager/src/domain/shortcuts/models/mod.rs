//! Shortcut value types.
//!
//! Pure data shared by config loading, matching and use cases.

use serde::{Deserialize, Serialize};

use super::handler::{parse_modifiers, to_keysym};

fn default_true() -> bool {
    true
}

/// One configured global shortcut: a key combo and the command it runs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Shortcut {
    pub(crate) id: usize,
    pub(crate) modifiers: String,
    pub(crate) key: String,
    pub(crate) command: String,
    #[serde(default = "default_true")]
    pub(crate) enabled: bool,
}

/// Modifier mask required for a shortcut to match a key press.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub(crate) struct Mods {
    pub(crate) shift: bool,
    pub(crate) ctrl: bool,
    pub(crate) alt: bool,
    pub(crate) win: bool,
}

/// A shortcut parsed into a keysym and its required modifier mask.
#[derive(Clone, Debug)]
pub(crate) struct ResolvedShortcut {
    pub(crate) mods: Mods,
    pub(crate) keysym: u32,
    pub(crate) command: String,
}

impl ResolvedShortcut {
    /// Parses a configured shortcut into a comparable keysym and modifier mask.
    ///
    /// # Arguments
    ///
    /// * `shortcut` - Raw configuration entry.
    ///
    /// # Returns
    ///
    /// `None` when the entry is disabled or its combo cannot be parsed.
    pub(crate) fn resolve(shortcut: &Shortcut) -> Option<Self> {
        if !shortcut.enabled {
            return None;
        }
        Some(Self {
            mods: parse_modifiers(&shortcut.modifiers)?,
            keysym: to_keysym(&shortcut.key)?,
            command: shortcut.command.clone(),
        })
    }
}
