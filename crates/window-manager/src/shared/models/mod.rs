//! Shared value models: geometry, keysyms, titlebar layout and window facts.
//!
//! These are plain data with pure rules; they read no config, spawn no processes
//! and touch no window-system protocol.

#[allow(unused_imports)]
pub(crate) mod geometry;
pub(crate) mod keysyms;
pub(crate) mod ports;
pub(crate) mod shortcut;
pub(crate) mod titlebar;
pub(crate) mod window;

#[allow(unused_imports)]
pub(crate) use ports::{ConfigError, LaunchError};
#[allow(unused_imports)]
pub(crate) use titlebar::TitlebarLayout;
#[allow(unused_imports)]
pub(crate) use window::{WindowMetadata, WindowVisualState};
