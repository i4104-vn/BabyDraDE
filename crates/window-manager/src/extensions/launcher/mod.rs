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

use common::constants::launcher::{
    BACKDROP_WIDTH_MIN, BACKDROP_WIDTH_RATIO, HAIRLINE, ICON, ICON_SELECTED, INDICATOR_HEIGHT,
    INDICATOR_WIDTH, LABEL_FONT_SIZE, LABEL_GAP, LABEL_HEIGHT, LABEL_MAX_TEXT, LABEL_MAX_WIDTH,
    LABEL_PAD, PANEL_CORNER_RADIUS, PANEL_FILLET_RADIUS, PANEL_PAD, PANEL_SURFACE, PANEL_WIDTH,
    PANEL_X, SELECTION_SIZE, SHADOW_PAD, SLOT, VISIBLE_SLOTS,
};
use gtk4::{gdk_pixbuf::Pixbuf, gio::AppInfo};

/// Physical region covered by an off-screen buffer: `(x, y, width, height)`.
pub(super) type BufferRegion = (i32, i32, i32, i32);
pub(super) type CacheKey = (usize, Option<usize>, u64, BufferRegion);
/// Backdrop cache key: physical size plus the presence-animation tick.
pub(super) type BackdropKey = ((i32, i32), u64);

pub(super) struct LauncherApp {
    pub(super) name: String,
    pub(super) info: AppInfo,
    pub(super) icon: Option<Pixbuf>,
}
