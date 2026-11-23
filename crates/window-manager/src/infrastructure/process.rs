//! Process launching driver (Frameworks & Drivers layer).
//!
//! Implements the `LaunchCommand` and `AutostartLauncher` ports declared by the
//! application layer.

use std::ffi::OsStr;

use tracing::{error, warn};

use crate::application::ports::AutostartLauncher;

/// Spawns a configured command through `sh -c` in a detached thread so shell
/// syntax keeps working without blocking the compositor's event loop.
pub(crate) fn spawn_detached(command: String) {
    if let Err(e) = std::thread::Builder::new()
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
    {
        error!("failed to spawn keymap command thread: {}", e);
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
