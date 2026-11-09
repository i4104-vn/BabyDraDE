//! BabyDraDE Keymap Daemon — Global hotkey handler.

pub mod daemon;
pub mod keyboard;
pub mod mapping;

/// Runs the keymap daemon.
pub fn run() {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build tokio runtime")
        .block_on(daemon::run());
}
