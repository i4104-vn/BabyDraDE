//! Compositor-owned application launcher: a docked vertical notch on the left
//! edge, styled after the topbar notch (near-black surface, hairline white border,
//! smooth concave fillets transition to the edge, rounded outer corners). Shown
//! via its shortcut and navigated with arrow keys, the wheel or the pointer.
//!
//! The module is split by responsibility: [`layout`] owns geometry, [`draw`]
//! owns Cairo drawing, [`buffer`] owns off-screen pixel work, and [`state`]
//! owns selection/input state.

mod buffer;
mod draw;
mod layout;
mod state;

pub(crate) use state::LauncherState;

use gtk4::{gdk_pixbuf::Pixbuf, gio::AppInfo};

pub(super) const PANEL_X: f64 = 0.0;
pub(super) const PANEL_WIDTH: f64 = 72.0;
pub(super) const PANEL_CORNER_RADIUS: f64 = 24.0;
pub(super) const PANEL_FILLET_RADIUS: f64 = 20.0;
pub(super) const PANEL_PAD: f64 = 24.0;
pub(super) const ICON: f64 = 30.0;
pub(super) const ICON_SELECTED: f64 = 42.0;
pub(super) const SLOT: f64 = 50.0;
pub(super) const VISIBLE_SLOTS: usize = 9;
pub(super) const SELECTION_SIZE: f64 = 48.0;
pub(super) const INDICATOR_WIDTH: f64 = 3.0;
pub(super) const INDICATOR_HEIGHT: f64 = 16.0;
pub(super) const LABEL_GAP: f64 = 10.0;
pub(super) const LABEL_HEIGHT: f64 = 32.0;
pub(super) const LABEL_PAD: f64 = 12.0;
pub(super) const LABEL_FONT_SIZE: f64 = 13.5;
pub(super) const LABEL_MAX_TEXT: f64 = 190.0;
pub(super) const LABEL_MAX_WIDTH: f64 = LABEL_MAX_TEXT + LABEL_PAD * 2.0;
/// Extra space around the capsule so its soft shadow is not clipped.
pub(super) const SHADOW_PAD: f64 = 30.0;
pub(super) const MOVE_ANIM_MS: u64 = 220;
pub(super) const SCROLL_ANIM_MS: u64 = 180;

pub(super) const BACKDROP_WIDTH_MIN: f64 = 600.0;
pub(super) const BACKDROP_WIDTH_RATIO: f64 = 0.60;

// Matches the topbar capsules (`rgb(14, 14, 18)` with a 12% white hairline).
pub(super) const PANEL_SURFACE: (f64, f64, f64, f64) =
    (14.0 / 255.0, 14.0 / 255.0, 18.0 / 255.0, 0.96);
pub(super) const HAIRLINE: (f64, f64, f64, f64) = (1.0, 1.0, 1.0, 0.12);

/// Physical region covered by an off-screen buffer: `(x, y, width, height)`.
pub(super) type BufferRegion = (i32, i32, i32, i32);
pub(super) type CacheKey = (usize, Option<usize>, u64, BufferRegion);

pub(super) struct LauncherApp {
    pub(super) name: String,
    pub(super) info: AppInfo,
    pub(super) icon: Option<Pixbuf>,
}
