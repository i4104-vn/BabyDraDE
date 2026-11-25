//! Use case: resolve and dispatch configurable global shortcuts.
//!
//! The use case owns no config files and spawns no processes by itself. The
//! composition root injects both through ports that carry structured errors,
//! which keeps this layer dependent on `domain` only and makes the ports
//! straightforward to fake in tests. The resulting service is passed by
//! reference to the components that need it instead of living in a global.

use std::sync::{Arc, RwLock};

use crate::shared::ports::{CommandLauncher, ShortcutRepository};
use crate::core::shortcuts::domain::{self, Mods, Shortcut};

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
        domain::command_for(mods, keysym, &guard)
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

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::sync::Mutex;

    use super::ShortcutService;
    use crate::shared::ports::{CommandLauncher, ConfigError, LaunchError, ShortcutRepository};
    use crate::shared::types::keysyms::KEY_e;
    use crate::core::shortcuts::domain::{Mods, Shortcut};

    fn shortcut(command: &str) -> Shortcut {
        Shortcut {
            id: 1,
            modifiers: "W".into(),
            key: "e".into(),
            command: command.into(),
            enabled: true,
        }
    }

    fn mods() -> Mods {
        Mods {
            win: true,
            ..Mods::default()
        }
    }

    struct FakeRepository {
        result: Mutex<Result<Vec<Shortcut>, ConfigError>>,
        loads: AtomicUsize,
    }

    impl FakeRepository {
        fn returning(shortcuts: Vec<Shortcut>) -> Self {
            Self {
                result: Mutex::new(Ok(shortcuts)),
                loads: AtomicUsize::new(0),
            }
        }

        fn failing(message: &str) -> Self {
            Self {
                result: Mutex::new(Err(ConfigError {
                    message: message.into(),
                })),
                loads: AtomicUsize::new(0),
            }
        }

        fn set_result(&self, result: Result<Vec<Shortcut>, ConfigError>) {
            *self.result.lock().unwrap() = result;
        }

        fn load_count(&self) -> usize {
            self.loads.load(Ordering::SeqCst)
        }
    }

    impl ShortcutRepository for FakeRepository {
        fn load(&self) -> Result<Vec<Shortcut>, ConfigError> {
            self.loads.fetch_add(1, Ordering::SeqCst);
            self.result.lock().unwrap().clone()
        }
    }

    struct FakeLauncher {
        result: Result<(), LaunchError>,
        commands: Mutex<Vec<String>>,
    }

    impl Default for FakeLauncher {
        fn default() -> Self {
            Self {
                result: Ok(()),
                commands: Mutex::new(Vec::new()),
            }
        }
    }

    impl FakeLauncher {
        fn failing(message: &str) -> Self {
            Self {
                result: Err(LaunchError {
                    message: message.into(),
                }),
                ..Self::default()
            }
        }

        fn commands(&self) -> Vec<String> {
            self.commands.lock().unwrap().clone()
        }
    }

    impl CommandLauncher for FakeLauncher {
        fn launch(&self, command: &str) -> Result<(), LaunchError> {
            self.commands.lock().unwrap().push(command.to_string());
            self.result.clone()
        }
    }

    fn service_with(repository: Arc<FakeRepository>, launcher: Arc<FakeLauncher>) -> ShortcutService {
        ShortcutService::new(repository, launcher)
    }

    #[test]
    fn loads_initial_shortcuts_through_repository() {
        let repository = Arc::new(FakeRepository::returning(vec![shortcut("kitty")]));
        let service = service_with(repository.clone(), Arc::new(FakeLauncher::default()));

        assert_eq!(repository.load_count(), 1);
        assert_eq!(service.command_for(mods(), KEY_e).as_deref(), Some("kitty"));
    }

    #[test]
    fn initial_load_error_starts_with_an_empty_table() {
        let repository = Arc::new(FakeRepository::failing("no keymap"));
        let service = service_with(repository.clone(), Arc::new(FakeLauncher::default()));

        assert_eq!(repository.load_count(), 1);
        assert_eq!(service.command_for(mods(), KEY_e), None);
    }

    #[test]
    fn reload_replaces_the_table() {
        let repository = Arc::new(FakeRepository::returning(vec![shortcut("kitty")]));
        let service = service_with(repository.clone(), Arc::new(FakeLauncher::default()));

        repository.set_result(Ok(vec![shortcut("alacritty")]));
        service.reload();

        assert_eq!(repository.load_count(), 2);
        assert_eq!(
            service.command_for(mods(), KEY_e).as_deref(),
            Some("alacritty")
        );
    }

    #[test]
    fn reload_error_keeps_previous_table() {
        let repository = Arc::new(FakeRepository::returning(vec![shortcut("kitty")]));
        let service = service_with(repository.clone(), Arc::new(FakeLauncher::default()));

        repository.set_result(Err(ConfigError {
            message: "bad config".into(),
        }));
        service.reload();

        assert_eq!(repository.load_count(), 2);
        assert_eq!(service.command_for(mods(), KEY_e).as_deref(), Some("kitty"));
    }

    #[test]
    fn command_for_matches_only_the_configured_combo() {
        let service = service_with(
            Arc::new(FakeRepository::returning(vec![shortcut("kitty")])),
            Arc::new(FakeLauncher::default()),
        );

        assert_eq!(service.command_for(mods(), KEY_e).as_deref(), Some("kitty"));
        assert_eq!(service.command_for(Mods::default(), KEY_e), None);
    }

    #[test]
    fn launch_error_is_reported_without_panicking() {
        let launcher = Arc::new(FakeLauncher::failing("spawn failed"));
        let service = service_with(
            Arc::new(FakeRepository::returning(vec![shortcut("kitty")])),
            launcher.clone(),
        );

        let command = service.command_for(mods(), KEY_e).unwrap();
        service.launch(command);

        assert_eq!(launcher.commands(), vec!["kitty".to_string()]);

        // A failed launch does not disable the service.
        service.launch("second".into());
        assert_eq!(
            launcher.commands(),
            vec!["kitty".to_string(), "second".to_string()]
        );
    }
}
