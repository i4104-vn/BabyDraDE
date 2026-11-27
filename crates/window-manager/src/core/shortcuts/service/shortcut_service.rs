//! Use case: resolve and dispatch configurable global shortcuts.
//!
//! The use case owns no config files and spawns no processes by itself. The
//! composition root injects both through ports that carry structured errors,
//! which keeps this layer dependent on `domain` only and makes the ports
//! straightforward to fake in tests. The resulting service is passed by
//! reference to the components that need it instead of living in a global.
//!
//! Domain rules are reached through `crate::core::shortcuts` re-exports.

use std::sync::{Arc, RwLock};

use crate::core::shortcuts::{command_for, Mods, Shortcut};
use crate::shared::ports::{CommandLauncher, ShortcutRepository};

/// Injected shortcut service shared by the input pipeline and reload watcher.
pub(crate) struct ShortcutService {
    shortcuts: Arc<RwLock<Vec<Shortcut>>>,
    repository: Arc<dyn ShortcutRepository + Send + Sync>,
    launcher: Arc<dyn CommandLauncher + Send + Sync>,
}

impl ShortcutService {
    /// Creates a service from the config and launcher ports supplied by the
    /// composition root.
    ///
    /// # Arguments
    ///
    /// * `repository` - Port that loads the shortcut table.
    /// * `launcher` - Port that launches a matched command.
    ///
    /// # Returns
    ///
    /// The initialised service. An initial load error yields an empty table.
    pub(crate) fn new(
        repository: Arc<dyn ShortcutRepository + Send + Sync>,
        launcher: Arc<dyn CommandLauncher + Send + Sync>,
    ) -> Self {
        let initial = match repository.load() {
            Ok(shortcuts) => shortcuts,
            Err(e) => {
                tracing::warn!("cannot load initial shortcut table: {}", e.message);
                Vec::new()
            }
        };
        Self {
            shortcuts: Arc::new(RwLock::new(initial)),
            repository,
            launcher,
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
        command_for(mods, keysym, &guard)
    }

    /// Reloads the shortcut table through the injected config port.
    ///
    /// A failed load is logged and the previous snapshot is kept.
    pub(crate) fn reload(&self) {
        match self.repository.load() {
            Ok(shortcuts) => {
                *self.shortcuts.write().unwrap_or_else(|e| e.into_inner()) = shortcuts;
            }
            Err(e) => tracing::warn!(
                "kept previous shortcut table after load error: {}",
                e.message
            ),
        }
    }

    /// Executes a command through the injected launcher port.
    ///
    /// # Arguments
    ///
    /// * `command` - Command string from the matched shortcut.
    pub(crate) fn launch(&self, command: String) {
        if let Err(e) = self.launcher.launch(&command) {
            tracing::warn!(
                "failed to launch shortcut command {:?}: {}",
                command,
                e.message
            );
        }
    }
}
