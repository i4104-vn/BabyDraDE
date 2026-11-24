//! Compositor-owned application launcher: a docked vertical notch on the left
//! edge, styled after the topbar notch (near-black surface, hairline white border,
//! smooth concave fillets transition to the edge, rounded outer corners). Shown
//! via its shortcut and navigated with arrow keys, the wheel or the pointer.
//!
//! Geometry comes from `ui_kit::components::dock`; [`draw`] owns Cairo drawing
//! and [`buffer`] owns off-screen pixel work. Selection and input state are
//! held by the compositor adapter. [`search`] is the expanded overlay that takes
//! over as soon as the user types while the dock is open.

pub(crate) mod buffer;
pub(crate) mod render;
pub(crate) mod search;

pub(crate) use common::constants::launcher::{
    BACKDROP_WIDTH_MIN, BACKDROP_WIDTH_RATIO, LABEL_FONT_SIZE, LABEL_GAP, LABEL_HEIGHT,
    LABEL_MAX_TEXT, LABEL_MAX_WIDTH, LABEL_PAD, SHADOW_PAD, SLOT,
};
use gtk4::{gdk_pixbuf::Pixbuf, gio::AppInfo};

/// Physical region covered by an off-screen buffer: `(x, y, width, height)`.
pub(crate) type BufferRegion = (i32, i32, i32, i32);
/// Dock cache key: app count, selection, dock/search animation ticks, region
/// and the effective presence bits (so fade boundaries never reuse stale pixels).
pub(crate) type CacheKey = (usize, Option<usize>, u64, u64, BufferRegion, u64);
/// Backdrop cache key: physical size, both animation ticks and presence bits.
pub(crate) type BackdropKey = ((i32, i32), u64, u64, u64);

pub(crate) struct LauncherApp {
    pub(crate) name: String,
    pub(crate) info: AppInfo,
    pub(crate) icon: Option<Pixbuf>,
}
