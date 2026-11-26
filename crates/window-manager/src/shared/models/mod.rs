//! Shared value models: geometry, keysyms, titlebar layout and window facts.
//!
//! These are plain data with pure rules; they read no config, spawn no processes
//! and touch no window-system protocol.

pub(crate) mod geometry;
pub(crate) mod keysyms;
pub(crate) mod ports;
pub(crate) mod titlebar;
pub(crate) mod window;
