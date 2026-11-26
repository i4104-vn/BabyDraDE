//! Self-contained shell features: each owns its state, input handling,
//! layout and drawing in a single vertical slice.
//!
//! These are optional desktop overlays (launcher, Alt+Tab switcher, lock screen
//! and screenshot) and are kept separate from `rendering`, which only handles
//! system/window rendering and blur.

pub(crate) mod filedialog;
pub(crate) mod launcher;
pub(crate) mod lock;
pub(crate) mod screenshot;
pub(crate) mod switcher;

pub(crate) use switcher::SwitcherOverlay;
