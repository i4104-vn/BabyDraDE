use std::{ffi::OsString, process::Command};
use tracing::{info, warn};

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

pub fn spawn_new_terminal(socket_name: &std::ffi::OsStr) {
    let term = resolve_test_app(None);
    info!(
        "Spawning terminal '{:?}' with WAYLAND_DISPLAY={:?}",
        term, socket_name
    );

    if let Err(err) = Command::new(&term)
        .env("WAYLAND_DISPLAY", socket_name)
        .spawn()
    {
        warn!("Failed to spawn terminal '{:?}': {err}", term);
    }
}
