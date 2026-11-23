//! Process launching driver (Frameworks & Drivers layer).
//!
//! Implements the `CommandLauncher` and `AutostartLauncher` ports declared by
//! the application layer.

use std::ffi::OsStr;

use tracing::{error, warn};

use crate::application::ports::{AutostartLauncher, CommandLauncher, LaunchError};

/// Spawns a configured command through `sh -c` in a detached thread so shell
/// syntax keeps working without blocking the compositor's event loop.
///
/// Failures to start the worker are logged here; callers that need to react
/// to a failure use [`try_spawn_detached`] instead.
pub(crate) fn spawn_detached(command: String) {
    if let Err(message) = try_spawn_detached(command) {
        error!("failed to spawn keymap command thread: {}", message);
    }
}

/// Detached spawn used by the `CommandLauncher` port.
fn try_spawn_detached(command: String) -> Result<(), String> {
    std::thread::Builder::new()
        .name("babydra-keymap-command".to_string())
        .spawn(move || {
            match std::process::Command::new("sh")
                .arg("-c")
                .arg(&command)
                .status()
            {
                Ok(status) if status.success() => {}
                Ok(status) => warn!("keymap command {:?} exited with {}", command, status),
                Err(e) => error!("failed to run keymap command {:?}: {}", command, e),
            }
        })
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// Launches configured shortcut commands through the application port.
#[derive(Debug, Default)]
pub(crate) struct ShortcutCommandLauncher;

impl CommandLauncher for ShortcutCommandLauncher {
    fn launch(&self, command: &str) -> Result<(), LaunchError> {
        try_spawn_detached(command.to_string()).map_err(|message| LaunchError { message })
    }
}

/// Launches autostart commands on the compositor's Wayland display.
#[derive(Debug, Default)]
pub(crate) struct ProcessLauncher;

impl AutostartLauncher for ProcessLauncher {
    type Child = std::process::Child;

    fn spawn(&self, command: &str, wayland_display: &OsStr) -> Result<Self::Child, String> {
        std::process::Command::new("sh")
            .arg("-c")
            .arg(command)
            .env("WAYLAND_DISPLAY", wayland_display)
            .spawn()
            .map_err(|err| err.to_string())
    }
}
