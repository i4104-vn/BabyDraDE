//! Shell extensions layered on top of the core rendering pipeline.
//!
//! These are optional desktop features (lock screen, screenshot, window switcher, cursor)
//! and are kept separate from `rendering`, which only handles system/window
//! rendering and blur.

pub(crate) mod clipboard;
pub mod cursor;
pub(crate) mod lock;
pub mod screenshot;
pub mod switcher;

pub use cursor::Cursor;
pub use switcher::SwitcherOverlay;
