//! Alt+Tab switcher state, overlay and geometry models.

pub(crate) use geometry::{Rect, SwitcherLayout};
pub(crate) use overlay::{RenderKey, SwitcherOverlay};
pub(crate) use switcher::{step_target, AltTabEntry, AltTabSwitcher};

pub(crate) mod geometry;
pub(crate) mod overlay;
pub(crate) mod switcher;
mod switcher_tests;
