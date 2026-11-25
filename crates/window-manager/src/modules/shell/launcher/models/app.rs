//! Launcher application and cache-key models.

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
