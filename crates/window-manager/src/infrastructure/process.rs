//! Process launching driver (Frameworks & Drivers layer).
//!
//! Implements the `LaunchCommand` port declared by the application layer.

use tracing::{error, warn};

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
