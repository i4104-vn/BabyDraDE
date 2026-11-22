//! Shell extensions layered on top of the core rendering pipeline.
//!
//! These are optional desktop features (lock screen, screenshot, window switcher)
//! and are kept separate from `rendering`, which only handles system/window
//! rendering and blur.

pub(crate) mod clipboard;
pub(crate) mod launcher;
pub(crate) mod lock;
pub(crate) mod screenshot;
pub(crate) mod switcher;

pub(crate) use switcher::SwitcherOverlay;
