use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::sync::Mutex;

use super::ShortcutService;
use crate::core::shortcuts::{Mods, Shortcut};
use crate::shared::ports::{CommandLauncher, ConfigError, LaunchError, ShortcutRepository};
use crate::shared::types::keysyms::KEY_e;

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
