//! Use case: resolve and dispatch configurable global shortcuts.
//!
//! The use case owns no config files and spawns no processes by itself. The
//! composition root injects both through plain function ports, which keeps
//! this layer dependent on `domain` only. The resulting service is passed by
//! reference to the components that need it instead of living in a global.

use std::sync::{Arc, RwLock};

use super::ports::{LaunchCommand, LoadShortcuts};
use crate::domain::shortcuts::{self, Mods, Shortcut};

/// Injected shortcut service shared by the input pipeline and reload watcher.
pub(crate) struct ShortcutService {
    shortcuts: Arc<RwLock<Vec<Shortcut>>>,
    load: LoadShortcuts,
    launch: LaunchCommand,
}

impl ShortcutService {
    /// Creates a service from the config and launcher ports supplied by the
    /// composition root.
    ///
    /// # Arguments
    ///
    /// * `load` - Port that loads the shortcut table.
    /// * `launch` - Port that launches a matched command.
    ///
    /// # Returns
    ///
    /// The initialised service.
    pub(crate) fn new(load: LoadShortcuts, launch: LaunchCommand) -> Self {
        Self {
            shortcuts: Arc::new(RwLock::new(load())),
            load,
            launch,
        }
    }

    /// Returns the shared shortcut table used by the reload watcher.
    pub(crate) fn shared(&self) -> &Arc<RwLock<Vec<Shortcut>>> {
        &self.shortcuts
    }

    /// Finds the command bound to a key press.
    ///
    /// # Arguments
    ///
    /// * `mods` - Active modifier mask.
    /// * `keysym` - Layout-independent keysym.
    ///
    /// # Returns
    ///
    /// The configured command, or `None` when nothing matches.
    pub(crate) fn command_for(&self, mods: Mods, keysym: u32) -> Option<String> {
        let guard = self.shortcuts.read().unwrap_or_else(|e| e.into_inner());
        shortcuts::command_for(mods, keysym, &guard)
    }

    /// Reloads the shortcut table through the injected config port.
    pub(crate) fn reload(&self) {
        *self
            .shortcuts
            .write()
            .unwrap_or_else(|e| e.into_inner()) = (self.load)();
    }

    /// Executes a command through the injected launcher port.
    ///
    /// # Arguments
    ///
    /// * `command` - Command string from the matched shortcut.
    pub(crate) fn launch(&self, command: String) {
        (self.launch)(command);
    }
}
