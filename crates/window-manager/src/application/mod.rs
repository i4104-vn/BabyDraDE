//! Application layer — use cases and the ports they depend on.
//!
//! Use cases own decisions (which command to run, which gesture maps to which
//! window action, what geometry a maximized window gets). They depend only on
//! `domain` and on the ports declared in [`ports`]; all framework effects are
//! injected by the composition root.

pub(crate) mod autostart;
pub(crate) mod clipboard;
pub(crate) mod gestures;
pub(crate) mod ports;
pub(crate) mod shortcuts;
pub(crate) mod switcher;
pub(crate) mod window_action;
pub(crate) mod windows;
