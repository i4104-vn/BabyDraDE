//! Screenshot background cache construction and blitting.

use gtk4::gdk::prelude::GdkCairoContextExt;

use super::super::{LayerCache, SharedCache};
use crate::modules::shell::screenshot::editor::models::EditorState;

/// Renders only the background screenshot into the cache surface. Rebuilt on
/// canvas resize, scale-factor change or editor state invalidation.
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
    if let Err(err) = cr.save() {
        tracing::warn!("cannot save cache context: {err}");
        return None;
    }
    if width > 0 && height > 0 && (bg_w != width as f64 || bg_h != height as f64) {
        cr.scale(width as f64 / bg_w, height as f64 / bg_h);
    }
    cr.set_source_pixbuf(&s.bg_pixbuf, 0.0, 0.0);
    cr.source().set_filter(cairo::Filter::Best);
    let painted = cr.paint();
    if let Err(err) = cr.restore() {
        tracing::warn!("cannot restore cache context: {err}");
    }
    if let Err(err) = painted {
        tracing::warn!("cannot paint cached background: {err}");
        return None;
    }

    Some(surface)
}

/// Refreshes the cached screenshot when the canvas size, scale or editor
/// generation changes.
pub(super) fn refresh_cache(
    s: &EditorState,
    width: i32,
    height: i32,
    scale: i32,
    cache: &SharedCache,
) {
    let mut cache_ref = cache.borrow_mut();
    let stale = match cache_ref.as_ref() {
        Some(c) => {
            c.width != width
                || c.height != height
                || c.scale != scale
                || c.version != s.render_version
        }
        None => true,
    };
    if stale {
        if let Some(surface) = render_cache_surface(s, width, height, scale) {
            *cache_ref = Some(LayerCache {
                width,
                height,
                scale,
                version: s.render_version,
                surface,
            });
        }
    }
}

/// Blits the cached background 1:1 in device pixels for maximum sharpness.
pub(super) fn blit_cache(cr: &cairo::Context, cache: &SharedCache) {
    let cache_ref = cache.borrow();
    if let Some(c) = cache_ref.as_ref() {
        if let Err(err) = cr.save() {
            tracing::warn!("cannot save context for cached background: {err}");
            return;
        }
        cr.scale(1.0 / c.scale as f64, 1.0 / c.scale as f64);
        let blit = cr.set_source_surface(&c.surface, 0.0, 0.0).and_then(|()| {
            cr.source().set_filter(cairo::Filter::Nearest);
            cr.paint()
        });
        if let Err(err) = cr.restore() {
            tracing::warn!("cannot restore context after cached background: {err}");
        }
        if let Err(err) = blit {
            tracing::warn!("cannot blit cached background: {err}");
        }
    }
}
