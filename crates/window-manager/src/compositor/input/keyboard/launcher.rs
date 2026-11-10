//! Terminal application launcher and executable resolution.

use std::{ffi::OsString, process::Command};
use tracing::{info, warn};

/// List of preferred terminal emulator binaries to search for in system PATH.
const TERMINAL_CANDIDATES: &[&str] = &[
    "foot",
    "alacritty",
    "kitty",
    "weston-terminal",
    "gnome-terminal",
    "xfce4-terminal",
    "konsole",
    "xterm",
    "urxvt",
    "st",
];

/// Resolves the terminal emulator executable to launch.
///
/// Priority order:
/// 1. Explicit application name passed via CLI argument.
/// 2. `BABYDRADE_TEST_APP` environment variable.
/// 3. First existing candidate from [`TERMINAL_CANDIDATES`] located in the current `PATH`.
/// 4. Fallback to `"foot"`.
///
/// # Arguments
///
/// * `explicit` - Optional explicitly specified binary name or path.
///
/// # Returns
///
/// Returns an [`OsString`] with the binary name or path to execute.
pub fn resolve_test_app(explicit: Option<OsString>) -> OsString {
    if let Some(app) = explicit {
        return app;
    }
    if let Some(app) = std::env::var_os("BABYDRADE_TEST_APP") {
        return app;
    }

    if let Some(paths) = std::env::var_os("PATH") {
        for candidate in TERMINAL_CANDIDATES {
            for dir in std::env::split_paths(&paths) {
                if dir.join(candidate).is_file() {
                    return OsString::from(candidate);
                }
            }
        }
    }

    OsString::from("foot")
}

/// Spawns a new terminal emulator window as a child process with Wayland environment variables set.
///
/// # Arguments
///
/// * `socket_name` - The active Wayland socket name (e.g. `"wayland-0"`) used to set `WAYLAND_DISPLAY`.
pub fn spawn_new_terminal(socket_name: &std::ffi::OsStr) {
    let term = resolve_test_app(None);
    info!(
        "Spawning terminal '{:?}' with WAYLAND_DISPLAY={:?}",
        term, socket_name
    );

    if let Err(err) = Command::new(&term)
        .env("WAYLAND_DISPLAY", socket_name)
        .env("ELECTRON_OZONE_PLATFORM_HINT", "auto")
        .env("MOZ_ENABLE_WAYLAND", "1")
        .env("QT_QPA_PLATFORM", "wayland;xcb")
        .env("GDK_BACKEND", "wayland,x11,*")
        .env("XDG_CURRENT_DESKTOP", "BabyDra")
        .env("XDG_SESSION_DESKTOP", "BabyDra")
        .env("XDG_SESSION_TYPE", "wayland")
        .spawn()
    {
        warn!("Failed to spawn terminal '{:?}': {err}", term);
    }
}

