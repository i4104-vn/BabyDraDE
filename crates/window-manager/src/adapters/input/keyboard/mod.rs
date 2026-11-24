//! Keyboard input processing, focus handling, and global hotkeys.

pub(crate) mod focus;
pub(crate) mod shortcuts;

mod handler;

pub(crate) use focus::cycle_window_focus;
pub(crate) use handler::handle_keyboard;
