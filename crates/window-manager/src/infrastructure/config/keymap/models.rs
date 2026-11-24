//! Keymap configuration models.

use std::{collections::HashMap, sync::{Arc, RwLock}};

use crate::domain::shortcuts::Shortcut;
use serde::Deserialize;

/// Shortcut table shared with the hot-reload watcher.
pub(crate) type SharedShortcuts = Arc<RwLock<Vec<Shortcut>>>;

#[derive(Deserialize, Default)]
pub(super) struct ConfigFile {
    #[serde(default)]
    pub(super) shortcuts: HashMap<String, String>,
}

