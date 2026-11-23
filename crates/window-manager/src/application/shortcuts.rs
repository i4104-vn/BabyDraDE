//! Use case: resolve and dispatch configurable global shortcuts.
//!
//! The use case owns no config files and spawns no processes by itself. The
//! composition root injects both through plain function ports, which keeps
//! this layer dependent on `domain` only.

use std::sync::{Arc, OnceLock, RwLock};

use super::ports::{LaunchCommand, LoadShortcuts};
use crate::domain::shortcuts::{self, Mods, Shortcut};

/// Process-wide registry of configured shortcuts.
pub(crate) struct ShortcutRegistry {
    shortcuts: Arc<RwLock<Vec<Shortcut>>>,
    load: LoadShortcuts,
    launch: LaunchCommand,
}

static REGISTRY: OnceLock<ShortcutRegistry> = OnceLock::new();

impl ShortcutRegistry {
    /// Initialises the process-wide registry with injected adapters.
    ///
    /// # Arguments
    ///
    /// * `load` - Port that loads the shortcut table.
    /// * `launch` - Port that launches a matched command.
    ///
    /// # Returns
    ///
    /// The initialised registry.
    pub(crate) fn init(load: LoadShortcuts, launch: LaunchCommand) -> &'static Self {
        REGISTRY.get_or_init(|| Self {
            shortcuts: Arc::new(RwLock::new(load())),
            load,
            launch,
        })
    }

    /// Returns the registry initialised by the composition root.
    ///
    /// # Panics
    ///
    /// Panics if the registry has not been initialised.
    pub(crate) fn get() -> &'static Self {
        REGISTRY
            .get()
            .expect("shortcut registry must be initialised by the composition root")
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
        let guard = self.shortcuts.read().unwrap();
        shortcuts::command_for(mods, keysym, &guard)
    }

    /// Reloads the shortcut table through the injected config port.
    pub(crate) fn reload(&self) {
        *self.shortcuts.write().unwrap() = (self.load)();
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
