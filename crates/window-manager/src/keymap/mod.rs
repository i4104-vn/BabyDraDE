//! Config-driven global shortcuts, evaluated inside the compositor input path.
//!
//! The compositor owns the input devices (libinput grabs them on a TTY), so a
//! separate evdev listener can never see key presses. Instead this module only
//! owns the configuration and exposes a matcher that
//! [`crate::input::keyboard`] calls for every key press.

pub mod config;
pub mod mapping;
pub mod matcher;

use std::sync::{Arc, OnceLock, RwLock};

use mapping::Shortcut;

/// Shortcut list shared with the hot-reload watcher.
pub type SharedShortcuts = Arc<RwLock<Vec<Shortcut>>>;

static SHORTCUTS: OnceLock<SharedShortcuts> = OnceLock::new();

/// Returns the process-wide shortcut table.
///
/// The config is loaded and the file watcher is started on first access.
pub fn shortcuts() -> &'static SharedShortcuts {
    SHORTCUTS.get_or_init(|| {
        let shared = config::load_shared();
        config::spawn_watcher(shared.clone());
        shared
    })
}

/// Reloads shortcuts from `~/.babydra/configs/keymap.toml` immediately.
pub fn reload() {
    if let Some(shared) = SHORTCUTS.get() {
        *shared.write().unwrap() = config::load();
    }
}

/// Spawns a configured command through `sh -c` in a detached thread so shell
/// syntax keeps working without blocking the compositor's event loop.
pub fn spawn_command(command: String) {
    if let Err(e) = std::thread::Builder::new()
        .name("babydra-keymap-command".to_string())
        .spawn(move || {
            match std::process::Command::new("sh")
                .arg("-c")
                .arg(&command)
                .status()
            {
                Ok(status) if status.success() => {}
                Ok(status) => tracing::warn!("keymap command {:?} exited with {}", command, status),
                Err(e) => tracing::error!("failed to run keymap command {:?}: {}", command, e),
            }
        })
    {
        tracing::error!("failed to spawn keymap command thread: {}", e);
    }
}
