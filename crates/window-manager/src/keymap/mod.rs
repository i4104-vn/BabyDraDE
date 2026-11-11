//! Global evdev keymap owned by the window manager.

pub mod config;
mod daemon;
mod keyboard;
mod mapping;

/// Starts the global keymap daemon on the current thread.
pub fn run() {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build keymap runtime")
        .block_on(daemon::run());
}
