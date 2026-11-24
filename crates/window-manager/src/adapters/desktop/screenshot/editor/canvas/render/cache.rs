//! Screenshot background cache construction and blitting.

use gtk4::gdk::prelude::GdkCairoContextExt;

use super::super::models::{LayerCache, SharedCache};
use crate::adapters::desktop::screenshot::editor::models::EditorState;

/// Renders only the background screenshot into the cache surface. Rebuilt on
/// canvas resize or scale-factor change, never while logic state changes.
pub(super) fn render_cache_surface(
    s: &EditorState,
    width: i32,
    height: i32,
    scale: i32,
) -> Option<cairo::ImageSurface> {
    let surface =
        cairo::ImageSurface::create(cairo::Format::ARgb32, width * scale, height * scale).ok()?;
    let cr = cairo::Context::new(&surface).ok()?;
    cr.scale(scale as f64, scale as f64);

    let bg_w = s.bg_pixbuf.width() as f64;
    let bg_h = s.bg_pixbuf.height() as f64;
    cr.set_antialias(cairo::Antialias::Best);
    cr.save().unwrap();
    if width > 0 && height > 0 && (bg_w != width as f64 || bg_h != height as f64) {
        cr.scale(width as f64 / bg_w, height as f64 / bg_h);
    }
    cr.set_source_pixbuf(&s.bg_pixbuf, 0.0, 0.0);
    cr.source().set_filter(cairo::Filter::Best);
    cr.paint().unwrap();
    cr.restore().unwrap();

    Some(surface)
}

/// Refreshes the cached screenshot when the canvas size or scale changes.
pub(super) fn refresh_cache(
    s: &EditorState,
    width: i32,
    height: i32,
    scale: i32,
    cache: &SharedCache,
) {
    let mut cache_ref = cache.borrow_mut();
    let stale = match cache_ref.as_ref() {
        Some(c) => c.width != width || c.height != height || c.scale != scale,
        None => true,
    };
    if stale {
        if let Some(surface) = render_cache_surface(s, width, height, scale) {
            *cache_ref = Some(LayerCache {
                width,
                height,
                scale,
                surface,
            });
        }
    }
}

/// Blits the cached background 1:1 in device pixels for maximum sharpness.
pub(super) fn blit_cache(cr: &cairo::Context, cache: &SharedCache) {
    let cache_ref = cache.borrow();
    if let Some(c) = cache_ref.as_ref() {
        cr.save().unwrap();
        cr.scale(1.0 / c.scale as f64, 1.0 / c.scale as f64);
        cr.set_source_surface(&c.surface, 0.0, 0.0).unwrap();
        cr.source().set_filter(cairo::Filter::Nearest);
        cr.paint().unwrap();
        cr.restore().unwrap();
    }
}
