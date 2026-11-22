//! Compositor-owned application launcher: a docked vertical notch on the left
//! edge, styled after the topbar notch (near-black surface, hairline white border,
//! smooth concave fillets transition to the edge, rounded outer corners). Shown
//! via its shortcut and navigated with arrow keys, the wheel or the pointer.
//!
//! The module is split by responsibility: [`layout`] owns geometry, [`draw`]
//! owns Cairo drawing, [`buffer`] owns off-screen pixel work, and [`state`]
//! owns selection/input state.

pub(crate) mod buffer;
pub(crate) mod draw;

pub(crate) use common::constants::launcher::{
    BACKDROP_WIDTH_MIN, BACKDROP_WIDTH_RATIO, LABEL_FONT_SIZE, LABEL_GAP, LABEL_HEIGHT,
    LABEL_MAX_TEXT, LABEL_MAX_WIDTH, LABEL_PAD, SHADOW_PAD, SLOT,
};
use gtk4::{gdk_pixbuf::Pixbuf, gio::AppInfo};

/// Physical region covered by an off-screen buffer: `(x, y, width, height)`.
pub(crate) type BufferRegion = (i32, i32, i32, i32);
pub(crate) type CacheKey = (usize, Option<usize>, u64, BufferRegion);
/// Backdrop cache key: physical size plus the presence-animation tick.
pub(crate) type BackdropKey = ((i32, i32), u64);

pub(crate) struct LauncherApp {
    pub(crate) name: String,
    pub(crate) info: AppInfo,
    pub(crate) icon: Option<Pixbuf>,
}
